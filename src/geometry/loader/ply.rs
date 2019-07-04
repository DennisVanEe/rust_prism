// This file contains the code for loading PLY files.
// This PLY file loader is NOT designed to be very "general", but it should perform alright.

use ply_rs::ply;
use ply_rs::parser;

use crate::geometry::mesh::{Mesh, Triangle};

#[derive(Copy)]
enum Endianness { LITTLE, BIG, }

/// Given a path, this function will load a mesh. If it can't load the 
/// mesh from the PLY file, it will indicate why.
pub fn load_path(path: &str) -> Result<Mesh> {
    let file = std::fs::File::open(path)?;
    let mut file = std::io::BufReader::new(file);

    // A basic parser is required to read the head:
    let parser = parser::Parser::<ply::ply::DefaultElement>::new();
    let header = parser.read_header(&mut file)?;

    // Given the header, we can extract the information we want so that we can process it the way
    // that we want to. The library doesn't support an efficient way of storing the properties of the
    // vertex into seperate buffers:
    
    
}

// Assumes that the file is of type uchar and that the faces are uint, otherwise file is rejected as being
// too bloated for it's own good:
fn load_faces_bin(file: &mut std::io::BufReader, count: usize, encoding: Endianness) -> Result<Vec<Triangle>> {
    // How much data do we expect (assuming triangles, of course):
    for count in 1..=count {
        let listcnt_buf = [0u8; 1]; // largest possible number it could be
        file.read_exact(&mut listcnt_buf)?; // read the char value we are interested in:
        // Convert:
        let listcnt = listcnt_buf[0];
        if listcn != 3u8 {
            return Err("detected non-triangular face in the ply file");
        }

        // If it all went well, load the rest of the bytes:
        let element_buf = [0u8; 12]; // largest possible number for three faces
        file.read_exact(&must element_buf)?;

        let face = [
            match encoding {
                Endianness::BIG => u32::from_be(unsafe { std::mem::transmute::<[u8; 4], u32>(element_buf[0..4]) }),
                Endianness::LITTLE => u32::from_le(unsafe { std::mem::transmute::<[u8; 4], u32>(element_buf[0..4]) }),
            },
            match encoding {
                Endianness::BIG => u32::from_be(unsafe { std::mem::transmute::<[u8; 4], u32>(element_buf[4..8]) }),
                Endianness::LITTLE => u32::from_le(unsafe { std::mem::transmute::<[u8; 4], u32>(element_buf[4..8]) }),
            } ,
            match encoding {
                Endianness::BIG => u32::from_be(unsafe { std::mem::transmute::<[u8; 4], u32>(element_buf[8..12]) }),
                Endianness::LITTLE => u32::from_le(unsafe { std::mem::transmute::<[u8; 4], u32>(element_buf[8..12]) }),
            },
        ];

        
    }
}   

// Loads the indices (only supports triangles), if non-triangle
// face detected, cancel now:
fn load_faces_ascii(count: usize, ) -> Result<Vec<Triangle>> {

}

// Loads the properties of the vertices(poss, norms, tans, uvs):
fn load_faces_ascii() -> Result<(Vec<Vec3f>, Vec<Vec3f>, Vec<Vec3f>, Vec<Vec2f>)> {

}