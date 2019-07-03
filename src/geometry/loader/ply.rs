// This file contains the code for loading PLY files.
// This PLY file loader is NOT designed to be very "general", but it should perform alright.

use ply_rs as ply;

use crate::geometry::mesh::{Mesh, Triangle};

/// Given a path, this function will load a mesh. If it can't load the 
/// mesh from the PLY file, it will indicate why.
pub fn load_path(path: &str) -> Result<Mesh> {
    let file = std::fs::File::open(path)?;
    let mut file = std::io::BufReader::new(file);

    // A basic parser is required to read the head:
    let parser = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    let header = parser.read_header(&mut file)?;

    // Given the header, we can extract the information we want:
    
}