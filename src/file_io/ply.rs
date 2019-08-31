// This file contains the code for loading PLY files.
// This PLY file loader is NOT designed to be very "general", but it should perform alright.

// The PLY file works as follows:
// If custom tangents are defined, then normals must also be defined.

use ply_rs::{parser, ply};
use simple_error::{bail, try_with, SimpleResult};

use crate::geometry::mesh::{Mesh, Triangle};
use crate::math::vector::{Vec2, Vec3};
use crate::memory::util::transmute_vec;

use std::fs::File;
use std::io::BufReader;

// Implement for different combinations of properties:
#[repr(C)]
struct VertexPos {
    pos: Vec3<f32>,
}

#[repr(C)]
struct VertexPosUV {
    pos: Vec3<f32>,
    uv: Vec2<f32>,
}

#[repr(C)]
struct VertexPosNrm {
    pos: Vec3<f32>,
    nrm: Vec3<f32>,
}

#[repr(C)]
struct VertexPosNrmUV {
    pos: Vec3<f32>,
    nrm: Vec3<f32>,
    uv: Vec2<f32>,
}

#[repr(C)]
struct VertexPosNrmTan {
    pos: Vec3<f32>,
    nrm: Vec3<f32>,
    tan: Vec3<f32>,
}

#[repr(C)]
struct VertexPosNrmTanUV {
    pos: Vec3<f32>,
    nrm: Vec3<f32>,
    tan: Vec3<f32>,
    uv: Vec2<f32>,
}

// Only position information in this case:
impl ply::PropertyAccess for VertexPos {
    fn new() -> Self {
        let pos = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        VertexPos { pos }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.pos.x = v,
            ("y", ply::Property::Float(v)) => self.pos.y = v,
            ("z", ply::Property::Float(v)) => self.pos.z = v,
            _ => (),
        }
    }
}

impl ply::PropertyAccess for VertexPosUV {
    fn new() -> Self {
        let pos = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let uv = Vec2 { x: 0f32, y: 0f32 };
        VertexPosUV { pos, uv }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.pos.x = v,
            ("y", ply::Property::Float(v)) => self.pos.y = v,
            ("z", ply::Property::Float(v)) => self.pos.z = v,
            ("u", ply::Property::Float(v)) => self.uv.x = v,
            ("v", ply::Property::Float(v)) => self.uv.y = v,
            ("s", ply::Property::Float(v)) => self.uv.x = v,
            ("t", ply::Property::Float(v)) => self.uv.y = v,
            ("texture_u", ply::Property::Float(v)) => self.uv.x = v,
            ("texture_v", ply::Property::Float(v)) => self.uv.y = v,
            ("texture_s", ply::Property::Float(v)) => self.uv.x = v,
            ("texture_t", ply::Property::Float(v)) => self.uv.y = v,
            _ => (),
        }
    }
}

impl ply::PropertyAccess for VertexPosNrm {
    fn new() -> Self {
        let pos = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let nrm = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        VertexPosNrm { pos, nrm }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.pos.x = v,
            ("y", ply::Property::Float(v)) => self.pos.y = v,
            ("z", ply::Property::Float(v)) => self.pos.z = v,
            ("nx", ply::Property::Float(v)) => self.nrm.x = v,
            ("ny", ply::Property::Float(v)) => self.nrm.y = v,
            ("nz", ply::Property::Float(v)) => self.nrm.z = v,
            _ => (),
        }
    }
}

impl ply::PropertyAccess for VertexPosNrmUV {
    fn new() -> Self {
        let pos = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let nrm = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let uv = Vec2 { x: 0f32, y: 0f32 };
        VertexPosNrmUV { pos, nrm, uv }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.pos.x = v,
            ("y", ply::Property::Float(v)) => self.pos.y = v,
            ("z", ply::Property::Float(v)) => self.pos.z = v,
            ("nx", ply::Property::Float(v)) => self.nrm.x = v,
            ("ny", ply::Property::Float(v)) => self.nrm.y = v,
            ("nz", ply::Property::Float(v)) => self.nrm.z = v,
            ("u", ply::Property::Float(v)) => self.uv.x = v,
            ("v", ply::Property::Float(v)) => self.uv.y = v,
            ("s", ply::Property::Float(v)) => self.uv.x = v,
            ("t", ply::Property::Float(v)) => self.uv.y = v,
            ("texture_u", ply::Property::Float(v)) => self.uv.x = v,
            ("texture_v", ply::Property::Float(v)) => self.uv.y = v,
            ("texture_s", ply::Property::Float(v)) => self.uv.x = v,
            ("texture_t", ply::Property::Float(v)) => self.uv.y = v,
            _ => (),
        }
    }
}

impl ply::PropertyAccess for VertexPosNrmTan {
    fn new() -> Self {
        let pos = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let nrm = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let tan = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        VertexPosNrmTan { pos, nrm, tan }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.pos.x = v,
            ("y", ply::Property::Float(v)) => self.pos.y = v,
            ("z", ply::Property::Float(v)) => self.pos.z = v,
            ("nx", ply::Property::Float(v)) => self.nrm.x = v,
            ("ny", ply::Property::Float(v)) => self.nrm.y = v,
            ("nz", ply::Property::Float(v)) => self.nrm.z = v,
            ("tx", ply::Property::Float(v)) => self.tan.x = v,
            ("ty", ply::Property::Float(v)) => self.tan.y = v,
            ("tz", ply::Property::Float(v)) => self.tan.z = v,
            _ => (),
        }
    }
}

impl ply::PropertyAccess for VertexPosNrmTanUV {
    fn new() -> Self {
        let pos = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let nrm = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let tan = Vec3 {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };
        let uv = Vec2 { x: 0f32, y: 0f32 };
        VertexPosNrmTanUV { pos, nrm, tan, uv }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.pos.x = v,
            ("y", ply::Property::Float(v)) => self.pos.y = v,
            ("z", ply::Property::Float(v)) => self.pos.z = v,
            ("nx", ply::Property::Float(v)) => self.nrm.x = v,
            ("ny", ply::Property::Float(v)) => self.nrm.y = v,
            ("nz", ply::Property::Float(v)) => self.nrm.z = v,
            ("tx", ply::Property::Float(v)) => self.tan.x = v,
            ("ty", ply::Property::Float(v)) => self.tan.y = v,
            ("tz", ply::Property::Float(v)) => self.tan.z = v,
            ("u", ply::Property::Float(v)) => self.uv.x = v,
            ("v", ply::Property::Float(v)) => self.uv.y = v,
            ("s", ply::Property::Float(v)) => self.uv.x = v,
            ("t", ply::Property::Float(v)) => self.uv.y = v,
            ("texture_u", ply::Property::Float(v)) => self.uv.x = v,
            ("texture_v", ply::Property::Float(v)) => self.uv.y = v,
            ("texture_s", ply::Property::Float(v)) => self.uv.x = v,
            ("texture_t", ply::Property::Float(v)) => self.uv.y = v,
            _ => (),
        }
    }
}

// This is separately for the triangle:
impl ply::PropertyAccess for Triangle {
    fn new() -> Self {
        Triangle {
            indices: [0u32, 0u32, 0u32],
        }
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("vertex_indices", ply::Property::ListInt(i)) => {
                // Make sure that it is a triangular mesh:
                if i.len() == 3 {
                    self.indices = unsafe {
                        [
                            *i.get_unchecked(0) as u32,
                            *i.get_unchecked(1) as u32,
                            *i.get_unchecked(2) as u32,
                        ]
                    };
                } // TODO: figure out what to do here
            }
            ("vertex_indices", ply::Property::ListUInt(i)) => {
                // Make sure that it is a triangular mesh:
                if i.len() == 3 {
                    self.indices = unsafe {
                        [
                            *i.get_unchecked(0),
                            *i.get_unchecked(1),
                            *i.get_unchecked(2),
                        ]
                    };
                } // TODO: figure out what to do here
            }
            _ => (),
        }
    }
}

/// Given a path, this function will load a mesh from a given PLY file.
/// It is important to note that this is not a general PLY file loader, it will only support
/// loading PLY files formatted in a specific way (though, most PLY files created by normal
/// 3D software should work).
pub fn load_path(path: &str) -> SimpleResult<Mesh> {
    let file = try_with!(File::open(path), "problem when opening ply file: {}", path);
    let mut file = BufReader::new(file);

    // A basic parser is required to read the head:
    let triangle_parser = parser::Parser::<Triangle>::new();
    let header = try_with!(
        triangle_parser.read_header(&mut file),
        "problem when parsing header of ply file: {}",
        path
    );

    // Given the header, we can now begin the process of parsing the header and getting the necessary
    // data from it:

    let mut has_pos = [false, false, false];
    let mut has_nrm = [false, false, false];
    let mut has_tan = [false, false, false];
    let mut has_uv = [false, false];
    let mut has_indices = false;

    for (_, element) in &header.elements {
        if element.name == "vertex" {
            // Loop over the elements of a vertex to see what we have:
            // We just ignore any properties we don't care about:
            for (_, property) in &element.properties {
                match (property.name.as_ref(), &property.data_type) {
                    ("x", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_pos[0] = true,
                    ("y", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_pos[1] = true,
                    ("z", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_pos[2] = true,
                    ("nx", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_nrm[0] = true,
                    ("ny", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_nrm[1] = true,
                    ("nz", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_nrm[2] = true,
                    ("tx", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_tan[0] = true,
                    ("ty", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_tan[1] = true,
                    ("tz", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_tan[2] = true,
                    ("u", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_uv[0] = true,
                    ("v", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_uv[1] = true,
                    ("s", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_uv[0] = true,
                    ("t", ply::PropertyType::Scalar(ply::ScalarType::Float)) => has_uv[1] = true,
                    ("texture_u", ply::PropertyType::Scalar(ply::ScalarType::Float)) => {
                        has_uv[0] = true
                    }
                    ("texture_v", ply::PropertyType::Scalar(ply::ScalarType::Float)) => {
                        has_uv[1] = true
                    }
                    ("texture_s", ply::PropertyType::Scalar(ply::ScalarType::Float)) => {
                        has_uv[0] = true
                    }
                    ("texture_t", ply::PropertyType::Scalar(ply::ScalarType::Float)) => {
                        has_uv[1] = true
                    }
                    _ => (),
                }
            }
        } else if element.name == "face" {
            // Could potentially have two properties we care about:
            for (_, property) in &element.properties {
                match (property.name.as_ref(), &property.data_type) {
                    ("vertex_indices", ply::PropertyType::List(_, ply::ScalarType::Int)) => {
                        has_indices = true
                    }
                    ("vertex_indices", ply::PropertyType::List(_, ply::ScalarType::UInt)) => {
                        has_indices = true
                    }
                    _ => (),
                }
            }
        }
    }

    // Let's check what we have:
    let has_pos = has_pos[0] && has_pos[1] && has_pos[2];
    let has_nrm = has_nrm[0] && has_nrm[1] && has_nrm[2];
    let has_tan = has_tan[0] && has_tan[1] && has_tan[2];
    let has_uv = has_uv[0] && has_uv[1];
    let has_indices = has_indices;

    // If it doesn't have positions or indices, we have a bad file:
    if !has_pos || !has_indices {
        bail!("ply file is missing positions or indices");
    }

    if has_tan && !has_nrm {
        bail!("no normals defined when given tans");
    }

    let mut triangles = Vec::new();

    // Otherwise we do this ugly mess to get the correct version efficiently (somewhat):
    let vertices = if !has_nrm && !has_tan && !has_uv {
        let vertex_parser = parser::Parser::<VertexPos>::new();
        let mut vertices = Vec::new();
        for (_ignore_key, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertices = try_with!(
                        vertex_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                "face" => {
                    triangles = try_with!(
                        triangle_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                _ => (),
            }
        }
        // Convert it to just floats:
        unsafe { transmute_vec::<VertexPos, f32>(vertices) }
    } else if !has_nrm && !has_tan && has_uv {
        let vertex_parser = parser::Parser::<VertexPosUV>::new();
        let mut vertices = Vec::new();
        for (_ignore_key, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertices = try_with!(
                        vertex_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                "face" => {
                    triangles = try_with!(
                        triangle_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                _ => (),
            }
        }
        // Convert it to just floats:
        unsafe { transmute_vec::<VertexPosUV, f32>(vertices) }
    } else if has_nrm && !has_tan && !has_uv {
        let vertex_parser = parser::Parser::<VertexPosNrm>::new();
        let mut vertices = Vec::new();
        for (_ignore_key, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertices = try_with!(
                        vertex_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                "face" => {
                    triangles = try_with!(
                        triangle_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                _ => (),
            }
        }
        // Convert it to just floats:
        unsafe { transmute_vec::<VertexPosNrm, f32>(vertices) }
    } else if has_nrm && !has_tan && has_uv {
        let vertex_parser = parser::Parser::<VertexPosNrmUV>::new();
        let mut vertices = Vec::new();
        for (_ignore_key, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertices = try_with!(
                        vertex_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                "face" => {
                    triangles = try_with!(
                        triangle_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                _ => (),
            }
        }
        // Convert it to just floats:
        unsafe { transmute_vec::<VertexPosNrmUV, f32>(vertices) }
    } else if has_nrm && has_tan && !has_uv {
        let vertex_parser = parser::Parser::<VertexPosNrmTan>::new();
        let mut vertices = Vec::new();
        for (_ignore_key, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertices = try_with!(
                        vertex_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                "face" => {
                    triangles = try_with!(
                        triangle_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                _ => (),
            }
        }
        // Convert it to just floats:
        unsafe { transmute_vec::<VertexPosNrmTan, f32>(vertices) }
    } else {
        let vertex_parser = parser::Parser::<VertexPosNrmTanUV>::new();
        let mut vertices = Vec::new();
        for (_ignore_key, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertices = try_with!(
                        vertex_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                "face" => {
                    triangles = try_with!(
                        triangle_parser.read_payload_for_element(&mut file, &element, &header),
                        "problem parsing ply file"
                    );
                }
                _ => (),
            }
        }
        // Convert it to just floats:
        unsafe { transmute_vec::<VertexPosNrmTanUV, f32>(vertices) }
    };

    // Great! Now we can go ahead and construct our damn mesh:
    Ok(Mesh::new(triangles, vertices, has_nrm, has_tan, has_uv))
}
