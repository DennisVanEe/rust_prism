use crate::geometry::mesh::{Attribute, Mesh, Triangle};
use crate::math::vector::{Vec2, Vec3};
use crate::transform::{AnimatedTransform, Transform};

use rply;
use simple_error::{bail, SimpleResult};

use std::ffi::{CStr, CString};
use std::os::raw;
use std::ptr;

pub fn load_mesh(
    paths: &[(String, Option<Transform>)],
    transform: AnimatedTransform,
    id: u32,
) -> SimpleResult<Mesh> {
    // Go through and allocate some headers:
    let mut headers = Vec::with_capacity(paths.len());
    for (path, _) in paths {
        headers.push(parse_header(path)?);
    }
    // Calculate how much we should allocate in total for the mesh:
    let (num_vertices, num_triangles) = headers.iter().fold((0, 0), |(nv, nt), header| {
        (nv + header.num_vertices, nt + header.num_triangles)
    });

    // Allocate (without filling) the buffer:
    let mut pos = Vec::with_capacity(num_vertices);
    let mut triangles = Vec::with_capacity(num_triangles);
    unsafe {
        pos.set_len(num_vertices);
        triangles.set_len(num_triangles);
    }

    // Load the different attributes
    let mut triangle_start = 0;
    let mut vertex_start = 0;
    let mut attributes = Vec::with_capacity(headers.len());
    for (attribute_id, (&header, (_, transf))) in headers.iter().zip(paths.iter()).enumerate() {
        let triangle_slice = {
            let triangle_end = triangle_start + header.num_triangles;
            &mut triangles[triangle_start..triangle_end]
        };
        let pos_slice = {
            let vertex_end = triangle_start + header.num_vertices;
            &mut pos[vertex_start..vertex_end]
        };

        attributes.push(load_attribute(
            header,
            pos_slice,
            triangle_slice,
            attribute_id as u32,
            triangle_start,
            vertex_start,
            *transf,
        )?);

        triangle_start += header.num_triangles;
        vertex_start += header.num_vertices;
    }

    Ok(Mesh::new(pos, triangles, attributes, transform, id))
}

extern "C" fn error_cb(_: rply::p_ply, message: *const raw::c_char) {
    let message = unsafe { CStr::from_ptr(message) };
    println!(
        "Error occured when parsing PLY file with message: \"{}\"",
        message.to_str().unwrap()
    );
}

extern "C" fn vec3_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = 0;
        let mut buffer_ptr = ptr::null_mut();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            return 0;
        }
        (
            item_index as usize,
            &mut *(buffer_ptr as *mut Vec<Vec3<f32>>),
        )
    };

    let index = unsafe {
        let mut index = 0;
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    unsafe {
        (*buffer.get_unchecked_mut(index))[item_index] =
            rply::ply_get_argument_value(argument) as f32
    }

    1
}

#[derive(Clone, Copy)]
struct MutSlicePtr<T: Sized> {
    ptr: *mut T,
    len: usize,
}

impl<T: Sized> MutSlicePtr<T> {
    fn new(t: &mut [T]) -> Self {
        MutSlicePtr {
            ptr: t.as_mut_ptr(),
            len: t.len(),
        }
    }

    unsafe fn to_slice<'a>(self) -> &'a mut [T] {
        std::slice::from_raw_parts_mut(self.ptr, self.len)
    }
}

// Needed a separate function to handle the case a slice is past into it:
extern "C" fn vec3_slice_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = 0;
        let mut buffer_ptr = ptr::null_mut();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            return 0;
        }
        let buffer = &*(buffer_ptr as *const MutSlicePtr<Vec3<f32>>);
        (item_index as usize, buffer.to_slice())
    };

    let index = unsafe {
        let mut index = 0;
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    unsafe {
        (*buffer.get_unchecked_mut(index))[item_index] =
            rply::ply_get_argument_value(argument) as f32
    }

    1
}

extern "C" fn vec2_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = 0;
        let mut buffer_ptr = ptr::null_mut();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        (
            item_index as usize,
            &mut *(buffer_ptr as *mut Vec<Vec2<f32>>),
        )
    };

    let index = unsafe {
        let mut index = 0;
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    unsafe {
        (*buffer.get_unchecked_mut(index))[item_index] =
            rply::ply_get_argument_value(argument) as f32
    }

    1
}

#[derive(Clone, Copy)]
pub struct TriangleParseInfo {
    buffer: MutSlicePtr<Triangle>,
    // Because the indexing is local to the attribute, we need to offset it by the number of vertices
    // that have come before it.
    offset: u32,
}

extern "C" fn index_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (buffer, offset) = unsafe {
        let mut buffer_ptr = ptr::null_mut();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, ptr::null_mut()) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        let info = &*(buffer_ptr as *const TriangleParseInfo);
        (info.buffer.to_slice(), info.offset)
    };

    let (num_indices, face_index) = unsafe {
        let mut num_indices = 0;
        let mut face_index = 0;
        if rply::ply_get_argument_property(
            argument,
            ptr::null_mut(),
            &mut num_indices,
            &mut face_index,
        ) == 0
        {
            return 0;
        }
        (num_indices as usize, face_index)
    };

    if num_indices != 3 {
        return 0;
    }

    if face_index < 0 {
        return 1;
    }

    let index = unsafe {
        let mut index = 0;
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    //let buff_index = (face_index as usize) + num_indices * index;
    unsafe {
        *buffer
            .get_unchecked_mut(index)
            .indices
            .get_unchecked_mut(face_index as usize) =
            (rply::ply_get_argument_value(argument) as u32) + offset;
    }

    1
}

#[derive(Clone, Copy)]
struct Header {
    file: rply::p_ply,
    num_vertices: usize,
    num_triangles: usize,
}

// Parses the header for a single header:
fn parse_header(path: &str) -> SimpleResult<Header> {
    let file = if let Ok(cstr_path) = CString::new(path) {
        unsafe { rply::ply_open(cstr_path.as_ptr(), Some(error_cb), 0, ptr::null_mut()) }
    } else {
        bail!("Could not convert the following to a valid path: {}", path)
    };

    unsafe {
        if rply::ply_read_header(file) == 0 {
            bail!("Couldn't parse header for PLY file at: {}", path);
        }
    }

    let mut element = ptr::null_mut();
    let mut num_vertices = 0;
    let mut num_triangles = 0;
    loop {
        element = unsafe { rply::ply_get_next_element(file, element) };
        if ptr::eq(element, ptr::null()) {
            break;
        }

        let mut element_name = ptr::null();
        let mut num_elements = 0;
        unsafe {
            rply::ply_get_element_info(element, &mut element_name, &mut num_elements);
        }

        unsafe {
            let element_name = CStr::from_ptr(element_name);
            if element_name.eq(CStr::from_bytes_with_nul_unchecked(b"vertex\0")) {
                num_vertices = num_elements as usize;
            } else if element_name.eq(CStr::from_bytes_with_nul_unchecked(b"face\0")) {
                num_triangles = num_elements as usize;
            }
        };
    }

    if num_vertices == 0 || num_triangles == 0 {
        bail!("No vertices or faces in the PLY file at: {}", path);
    }

    Ok(Header {
        file,
        num_vertices,
        num_triangles,
    })
}

// Loads a single mesh given the pos and triangle information to write:
fn load_attribute(
    header: Header,
    poss: &mut [Vec3<f32>],
    triangles: &mut [Triangle],
    attribute_id: u32,
    triangle_start: usize, // Where the triangle indices begin for the specific attribute
    vertex_start: usize,   // Where the vertex indices begin for the specific attribute
    transform: Option<Transform>,
) -> SimpleResult<Attribute> {
    let mut nrm = Vec::new();
    let mut tan = Vec::new();
    let mut uvs = Vec::new();

    // Get Position information:

    let poss_ptr = MutSlicePtr::new(poss);

    let has_x = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"x\0").as_ptr(),
            Some(vec3_slice_cb),
            (&poss_ptr as *const MutSlicePtr<Vec3<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_y = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"y\0").as_ptr(),
            Some(vec3_slice_cb),
            (&poss_ptr as *const MutSlicePtr<Vec3<f32>>) as *mut raw::c_void,
            1,
        )
    };
    let has_z = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"z\0").as_ptr(),
            Some(vec3_cb),
            (&poss_ptr as *const MutSlicePtr<Vec3<f32>>) as *mut raw::c_void,
            2,
        )
    };
    if has_x == 0 || has_y == 0 || has_z == 0 {
        bail!("No position information in the PLY file");
    }

    // Set the attribute values:
    for triangle in triangles.iter_mut() {
        // Material id is essentially 0:
        triangle.attribute_id = attribute_id;
    }

    // Get Normal information:

    let has_nx = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"nx\0").as_ptr(),
            Some(vec3_cb),
            (&mut nrm as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_ny = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"ny\0").as_ptr(),
            Some(vec3_cb),
            (&mut nrm as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            1,
        )
    };
    let has_nz = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"nz\0").as_ptr(),
            Some(vec3_cb),
            (&mut nrm as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            2,
        )
    };
    if has_nx != 0 && has_ny != 0 && has_nz != 0 {
        nrm.reserve_exact(header.num_vertices);
        unsafe {
            nrm.set_len(header.num_vertices);
        }
    }

    // Get Tangent information:

    let has_tx = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"tx\0").as_ptr(),
            Some(vec3_cb),
            (&mut tan as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_ty = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"ty\0").as_ptr(),
            Some(vec3_cb),
            (&mut tan as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            1,
        )
    };
    let has_tz = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"tz\0").as_ptr(),
            Some(vec3_cb),
            (&mut tan as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            2,
        )
    };
    if has_tx != 0 && has_ty != 0 && has_tz != 0 {
        tan.reserve_exact(header.num_vertices);
        unsafe {
            tan.set_len(header.num_vertices);
        }
    }

    // Get UV information:
    // Note that there are many naming schemes for this value:

    let has_u = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"u\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_v = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"v\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    let has_s = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"s\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_t = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"t\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    let has_texture_u = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_u\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_texture_v = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_v\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    let has_texture_s = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_s\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_texture_t = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_t\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    if (has_u != 0 && has_v != 0)
        || (has_s != 0 && has_t != 0)
        || (has_texture_u != 0 && has_texture_v != 0)
        || (has_texture_s != 0 && has_texture_t != 0)
    {
        uvs.reserve_exact(header.num_vertices);
        unsafe {
            uvs.set_len(header.num_vertices);
        }
    }

    // Get Index information:
    let triangle_info = TriangleParseInfo {
        buffer: MutSlicePtr::new(triangles),
        offset: vertex_start as u32,
    };

    let has_index = unsafe {
        rply::ply_set_read_cb(
            header.file,
            CStr::from_bytes_with_nul_unchecked(b"face\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"vertex_indices\0").as_ptr(),
            Some(index_cb),
            (&triangle_info as *const TriangleParseInfo) as *mut raw::c_void,
            0,
        )
    };
    if has_index == 0 {
        bail!("No face information in the PLY file");
    }

    if unsafe { rply::ply_read(header.file) } == 0 {
        bail!("Issue when reading PLY file");
    }

    // Go through and apply a transformatin if present:
    if let Some(transf) = transform {
        transf.points_f32(poss);
        transf.vectors_f32(&mut tan);
        transf.normals_f32(&mut nrm);
    }

    Ok(Attribute::new(
        triangle_start,
        triangles.len(),
        nrm,
        tan,
        uvs,
        attribute_id,
    ))
}
