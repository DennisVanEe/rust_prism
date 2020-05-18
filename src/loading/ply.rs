use crate::math::vector::{Vec2, Vec3};
use crate::mesh::{Mesh, Triangle};

use rply;
use simple_error::{bail, SimpleResult};

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::os::raw;
use std::ptr;

extern "C" fn error_cb(_: rply::p_ply, message: *const raw::c_char) {
    let err_msg = unsafe { CStr::from_ptr(message) };
    eprintln!(
        "PLY loading caused the following error: {}",
        err_msg.to_str().unwrap()
    );
}

extern "C" fn vec3_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = MaybeUninit::uninit().assume_init();
        let mut buffer_ptr = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        (item_index as usize, &mut *(buffer_ptr as *mut Vec<f32>))
    };

    let index = unsafe {
        let mut index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    let buff_index = 3 * index + item_index;
    unsafe {
        *buffer.get_unchecked_mut(buff_index) = rply::ply_get_argument_value(argument) as f32;
    }

    1
}

extern "C" fn vec2_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let (item_index, buffer) = unsafe {
        let mut item_index = MaybeUninit::uninit().assume_init();
        let mut buffer_ptr = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, &mut item_index) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        (item_index as usize, &mut *(buffer_ptr as *mut Vec<f32>))
    };

    let index = unsafe {
        let mut index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    let buff_index = 2 * index + item_index;
    unsafe {
        *buffer.get_unchecked_mut(buff_index) = rply::ply_get_argument_value(argument) as f32;
    }

    1
}

struct IndexBuffer {
    all_triangles: bool,
    buffer: Vec<Triangle>,
}

extern "C" fn index_cb(argument: rply::p_ply_argument) -> raw::c_int {
    let buffer = unsafe {
        let mut buffer_ptr = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_user_data(argument, &mut buffer_ptr, ptr::null_mut()) == 0 {
            // I think that the error_callback gets called so I don't have to log anything else
            return 0;
        }
        &mut *(buffer_ptr as *mut IndexBuffer)
    };

    let (num_indices, face_index) = unsafe {
        let mut num_indices = MaybeUninit::uninit().assume_init();
        let mut face_index = MaybeUninit::uninit().assume_init();
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
        buffer.all_triangles = false;
        return 0;
    }

    if face_index < 0 {
        return 1;
    }

    let index = unsafe {
        let mut index = MaybeUninit::uninit().assume_init();
        if rply::ply_get_argument_element(argument, ptr::null_mut(), &mut index) == 0 {
            return 0;
        }
        index as usize
    };

    //let buff_index = (face_index as usize) + num_indices * index;
    unsafe {
        *buffer
            .buffer
            .get_unchecked_mut(index)
            .indices
            .get_unchecked_mut(face_index as usize) = rply::ply_get_argument_value(argument) as u32;
    }

    1
}

pub fn load_mesh(path: &str) -> SimpleResult<Mesh> {
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

    let mut poss = Vec::new();
    let mut norms = Vec::new();
    let mut tans = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = IndexBuffer {
        all_triangles: true,
        buffer: Vec::new(),
    };

    // Get Position information:

    let has_x = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"x\0").as_ptr(),
            Some(vec3_cb),
            (&mut poss as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_y = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"y\0").as_ptr(),
            Some(vec3_cb),
            (&mut poss as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            1,
        )
    };
    let has_z = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"z\0").as_ptr(),
            Some(vec3_cb),
            (&mut poss as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            2,
        )
    };
    if has_x == 0 || has_y == 0 || has_z == 0 {
        bail!("No position information in the PLY file at: {}", path);
    }
    // Make sure to reserve space for one more:
    poss.reserve_exact(num_vertices + 1);
    unsafe {
        poss.set_len(num_vertices + 1);
    }

    // Get Normal information:

    let has_nx = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"nx\0").as_ptr(),
            Some(vec3_cb),
            (&mut norms as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_ny = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"ny\0").as_ptr(),
            Some(vec3_cb),
            (&mut norms as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            1,
        )
    };
    let has_nz = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"nz\0").as_ptr(),
            Some(vec3_cb),
            (&mut norms as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            2,
        )
    };
    if has_nx != 0 && has_ny != 0 && has_nz != 0 {
        norms.reserve_exact(num_vertices);
        unsafe {
            norms.set_len(num_vertices);
        }
    }

    // Get Tangent information:

    let has_tx = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"tx\0").as_ptr(),
            Some(vec3_cb),
            (&mut tans as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_ty = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"ty\0").as_ptr(),
            Some(vec3_cb),
            (&mut tans as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            1,
        )
    };
    let has_tz = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"tz\0").as_ptr(),
            Some(vec3_cb),
            (&mut tans as *mut Vec<Vec3<f32>>) as *mut raw::c_void,
            2,
        )
    };
    if has_tx != 0 && has_ty != 0 && has_tz != 0 {
        tans.reserve_exact(num_vertices + 1);
        unsafe {
            tans.set_len(num_vertices + 1);
        }
    }

    // Get UV information:
    // Note that there are many naming schemes for this value:

    let has_u = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"u\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_v = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"v\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    let has_s = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"s\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_t = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"t\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    let has_texture_u = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_u\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_texture_v = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_v\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            1,
        )
    };

    let has_texture_s = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"vertex\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"texture_s\0").as_ptr(),
            Some(vec2_cb),
            (&mut uvs as *mut Vec<Vec2<f32>>) as *mut raw::c_void,
            0,
        )
    };
    let has_texture_t = unsafe {
        rply::ply_set_read_cb(
            file,
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
        uvs.reserve_exact(num_vertices);
        unsafe {
            uvs.set_len(num_vertices);
        }
    }

    // Get Index information:

    let has_index = unsafe {
        rply::ply_set_read_cb(
            file,
            CStr::from_bytes_with_nul_unchecked(b"face\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(b"vertex_indices\0").as_ptr(),
            Some(index_cb),
            (&mut indices as *mut IndexBuffer) as *mut raw::c_void,
            0,
        )
    };
    if has_index == 0 {
        bail!("No face information in the PLY file at: {}", path);
    }

    indices.buffer.reserve_exact(num_triangles);
    unsafe {
        indices.buffer.set_len(num_triangles);
    }

    let result = unsafe { rply::ply_read(file) };

    // First check if there were any issues we can deduce:
    if indices.all_triangles {
        bail!("Non triangular face detected in PLY file at: {}", path)
    }

    if result == 0 {
        bail!("Issue when reading PLY file at: {}", path);
    }

    Ok(Mesh::new(indices.buffer, poss, norms, tans, uvs))
}
