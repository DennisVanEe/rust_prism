// This file contains the code for loading PLY files.
// This PLY file loader is NOT designed to be very "general", but it should perform alright.

use ply_rs::{ply, parser};
use simple_error::{SimpleResult, try_with, bail};

use std::io::{BufReader, BufRead, Read};
use std::fs::File;
use std::mem::transmute;

use crate::geometry::mesh::{Mesh, Triangle};
use crate::math::vector::{Vec3f, Vec2f};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Endianness { LITTLE, BIG, }

#[derive(Clone, Copy, PartialEq, Eq)]
enum VertexProp { POS, NRM, TAN, UV, NONE, }

/// Given a path, this function will load a mesh from a given PLY file.
/// It is important to note that this is not a general PLY file loader, it will only support
/// loading PLY files formatted in a specific way (though, most PLY files created by normal
/// 3D software should work).
pub fn load_path(path: &str) -> SimpleResult<Mesh> {
    let file = try_with!(File::open(path), "problem when opening ply file: {}", path);
    let mut file = BufReader::new(file);

    // A basic parser is required to read the head:
    let parser = parser::Parser::<ply::DefaultElement>::new();
    let header = try_with!(parser.read_header(&mut file), "problem when parsing header of ply file: {}", path);

    // Given the header, we can extract the information we want so that we can process it the way
    // that we want to. The library doesn't support an efficient way of storing the properties of the
    // vertex into seperate buffers:
    
    // Ok(())
}

// Assumes that the file is of type uchar and that the faces are uint, otherwise file is rejected as being
// too bloated for it's own good:
fn load_faces_bin(file: &mut BufReader::<File>, count: usize, encoding: Endianness) -> SimpleResult<Vec<Triangle>> {
    let mut result = Vec::with_capacity(count);
    
    // How much data do we expect (assuming triangles, of course):
    for i in 1..=count {
        let buffer = [0u8; 1]; // largest possible number it could be
        // Read the initial char byte:
        try_with!(file.read_exact(&mut buffer), "problem when reading ply file");

        let listcnt = buffer[0];
        if listcnt != 3u8 {
            bail!("non-triangular face detected in ply file")
        }

        // Read all of the face indices:
        let buffer = [0u8; 12];
        try_with!(file.read_exact(&mut buffer), "problem when reading ply file");

        // Perform a transmute:
        let unencoded_indices = unsafe { transmute::<[u8; 12], [u32; 3]>(buffer) };
        // Make suret he encoding is handled:
        let indices =  match encoding {
            Endianness::LITTLE => [ u32::from_le(unencoded_indices[0]), u32::from_le(unencoded_indices[1]), u32::from_le(unencoded_indices[2]) ],
            Endianness::BIG => [ u32::from_be(unencoded_indices[0]), u32::from_be(unencoded_indices[1]), u32::from_be(unencoded_indices[2]) ],
        };

        result.push(Triangle { indices });
    }

    return Ok(result);
}

// Load the faces, this won't be as efficient (it's recomended to use binary files):
fn load_faces_ascii(file: &mut BufReader::<File>, count: usize) -> SimpleResult<Vec<Triangle>> {
    let mut result = Vec::with_capacity(count);

    // Not the most efficient thing in the world. But it works and is maintainable.
    // Again, for something more efficient, use binary format:
    for i in 1..=count {
        let line = String::new();
        try_with!(file.read_line(&mut line), "problem when reading PLY file");

        // Check certain properties of the line:
        let line = line.split_ascii_whitespace().collect::<Vec<&str>>();
        if line.len() != 4usize {
            bail!("problem when parsing PLY file");
        }

        // Convert the string to the appropriate type:
        // We already checked the length of line, so to squeeze out a bit of performance,
        // we'll perform an unsafe get:
        let indices = unsafe { [
            try_with!(line.get_unchecked(1).parse::<u32>(), "problem when parsing PLY file"),
            try_with!(line.get_unchecked(2).parse::<u32>(), "problem when parsing PLY file"),
            try_with!(line.get_unchecked(3).parse::<u32>(), "problem when parsing PLY file"),
        ] };

        result.push(Triangle { indices });
    }

    Ok(result)
}

// Load the properties of the vertices (poss, norms, tans, uvs):
fn load_vertices_bin(file: &mut BufReader::<File>, count: usize, properties: &[VertexProp; 4], encoding: Endianness) -> SimpleResult<(Vec<Vec3f>, Vec<Vec3f>, Vec<Vec3f>, Vec<Vec2f>)> {
    let mut poss = Vec::new();
    let mut norms = Vec::new();
    let mut tans = Vec::new();
    let mut uvs = Vec::new();

    // Allocate memory before hand as appropriate:
    for &prop in properties {
        match prop {
            VertexProp::POS => poss.reserve(count),
            VertexProp::NRM => norms.reserve(count),
            VertexProp::TAN => tans.reserve(count),
            VertexProp::UV => uvs.reserve(count),
            VertexProp::NONE => break,
        };
    }

    // Now we go through the data:
    for i in 1..=count {
        for &prop in properties {
            if prop == VertexProp::NONE {
                // There are no other properties to worry about.
                break;
            } else if prop == VertexProp::UV {
                // Allocate buffer:
                let buffer = [0u8; 8];
                try_with!(file.read_exact(&mut buffer), "problem when reading PLY file");
                // Now we convert the data:
                let uv = unsafe { transmute::<[u8; 8], [u32; 2]>(buffer) };
                let uv = match encoding {
                    Endianness::LITTLE => [ u32::from_le(uv[0]), u32::from_le(uv[1]) ],
                    Endianness::BIG => [ u32::from_be(uv[0]), u32::from_be(uv[1]) ],
                };
                let uv = unsafe { transmute::<[u32; 2], Vec2f>(uv) };

                uvs.push(uv);
            } else {
                let buffer = [0u8; 12];
                try_with!(file.read_exact(&mut buffer), "problem when reading PLY file");
                // Now we convert the data:
                let vertex = unsafe { transmute::<[u8; 12], [u32; 3]>(buffer) };
                let vertex = match encoding {
                    Endianness::LITTLE => [ u32::from_le(vertex[0]), u32::from_le(vertex[1]), u32::from_le(vertex[2]) ],
                    Endianness::BIG => [ u32::from_be(vertex[0]), u32::from_be(vertex[1]), u32::from_be(vertex[2]) ],
                };
                let vertex = unsafe { transmute::<[u32; 3], Vec3f>(vertex) };

                match prop {
                    VertexProp::POS => poss.push(vertex),
                    VertexProp::NRM => norms.push(vertex),
                    VertexProp::TAN => tans.push(vertex),
                    // This should never happen:
                    VertexProp::UV | VertexProp::NONE => continue,
                }
            }
        }
    }

    Ok((poss, norms, tans, uvs))
}

// Loads the properties of the vertices(poss, norms, tans, uvs):
fn load_vertices_ascii(file: &mut BufReader::<File>, count: usize, properties: &[VertexProp; 4]) -> SimpleResult<(Vec<Vec3f>, Vec<Vec3f>, Vec<Vec3f>, Vec<Vec2f>)> {
    let mut poss = Vec::new();
    let mut norms = Vec::new();
    let mut tans = Vec::new();
    let mut uvs = Vec::new();

    // Allocate memory before hand as appropriate:
    let mut num_properties = 0usize;
    for &prop in properties {
        match prop {
            VertexProp::POS => { poss.reserve(count); num_properties += 3; },
            VertexProp::NRM => { norms.reserve(count); num_properties += 3; },
            VertexProp::TAN => { tans.reserve(count); num_properties += 3; },
            VertexProp::UV => { uvs.reserve(count); num_properties += 2; },
            VertexProp::NONE => break,
        };
    }

    for i in 1..=count {
        let line = String::new();
        try_with!(file.read_line(line), "problem when reading PLY file");

        // Check certain properties of the line:
        let line = line.split_ascii_whitespace().collect::<Vec<&str>>();
        if line.len() != num_properties {
            bail!("problem when parsing PLY file");
        }

        let
        for &prop in properties {
            if prop == VertexProp::NONE {
                break;
            } else {
                match prop {

                }
            }
        }
    }
}