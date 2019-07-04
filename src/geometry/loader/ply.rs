// This file contains the code for loading PLY files.
// This PLY file loader is NOT designed to be very "general", but it should perform alright.

use ply_rs::ply;
use ply_rs::parser;

use std::io::BufReader;
use std::io::Read;
use std::fs::File;

use crate::geometry::mesh::{Mesh, Triangle};

#[derive(Copy)]
enum Endianness { LITTLE, BIG, }

/// Given a path, this function will load a mesh. If it can't load the 
/// mesh from the PLY file, it will indicate why.
pub fn load_path(path: &str) -> Result<Mesh> {
    let file = File::open(path)?;
    let mut file = BufReader::new(file);

    // A basic parser is required to read the head:
    let parser = parser::Parser::<ply::DefaultElement>::new();
    let header = parser.read_header(&mut file)?;

    // Given the header, we can extract the information we want so that we can process it the way
    // that we want to. The library doesn't support an efficient way of storing the properties of the
    // vertex into seperate buffers:
    
    
}

// Converts byte array into three u32:
fn byte_to_three_u32(bytes: &[u8; 4 * 3], encoding: Endianness) -> [u32; 3] {
    match encoding {
        Endianness::BIG => [
            ((bytes[0] as u32) << 24) |
            ((bytes[1] as u32) << 16) |
            ((bytes[2] as u32) <<  8) |
            ((bytes[3] as u32) <<  0),

            ((bytes[4] as u32) << 24) |
            ((bytes[5] as u32) << 16) |
            ((bytes[6] as u32) <<  8) |
            ((bytes[7] as u32) <<  0),

            ((bytes[8] as u32) << 24) |
            ((bytes[9] as u32) << 16) |
            ((bytes[10] as u32) << 8) |
            ((bytes[11] as u32) << 0),
        ],

        Endianness::LITTLE => [
            ((bytes[0] as u32) <<  0) |
            ((bytes[1] as u32) <<  8) |
            ((bytes[2] as u32) << 16) |
            ((bytes[3] as u32) << 24),
            
            ((bytes[4] as u32) <<  0) |
            ((bytes[5] as u32) <<  8) |
            ((bytes[6] as u32) << 16) |
            ((bytes[7] as u32) << 24),

            ((bytes[8] as u32) <<  0)  |
            ((bytes[9] as u32) <<  8)  |
            ((bytes[10] as u32) << 16) |
            ((bytes[11] as u32) << 24),
        ]
    }
}

// Assumes that the file is of type uchar and that the faces are uint, otherwise file is rejected as being
// too bloated for it's own good:
fn load_faces_bin(file: &mut BufReader::<File>, count: usize, encoding: Endianness) -> Result<Vec<Triangle>, 'static str> {
    let mut result = Vec::with_capacity(count);
    
    // How much data do we expect (assuming triangles, of course):
    for count in 1..=count {
        let buffer = [0u8; 1]; // largest possible number it could be
        // Read all of the data
        match file.read_exact(&mut buffer) {
          Ok(_) => (),
          Err(e) => return e.to_string(),
        };
        
        // Convert:
        let listcnt = buffer[0];
        if listcnt != 3u8 {
            return Err(String::from_str("detected non-triangular face in the ply file"));
        }

        let buffer = [0u8; 4 * 3];
        file.read_exact(&mut buffer)?;

        let face = byte_to_three_u32(&buffer, encoding);
        result.push(Triangle { indices: face });
    }

    return Ok(result);
}

// Loads the indices (only supports triangles), if non-triangle
// face detected, cancel now:
fn load_faces_ascii(count: usize, ) -> Result<Vec<Triangle>> {

}

// Loads the properties of the vertices(poss, norms, tans, uvs):
fn load_faces_ascii() -> Result<(Vec<Vec3f>, Vec<Vec3f>, Vec<Vec3f>, Vec<Vec2f>)> {

}