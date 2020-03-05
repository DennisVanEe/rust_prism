use crate::memory;

use enum_map::{Enum, EnumMap};

pub mod beauty;

pub trait Pixel: Copy {
    // What finalize will output
    type FinalOutput: Copy;
    // The type of AOV, used to help indicate which buffer to access:
    const TypeID: PixelType;

    // Create an instance of the pixel in the initial state:
    fn zero() -> Self;
    // Sets the pixel to its "zero" state
    fn set_zero(&mut self);
    // How to add two pixels together:
    fn add(&mut self, p: &Self);
    // Outputs a "final" result for the current pixel:
    fn finalize(&self) -> Self::FinalOutput;
}

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum PixelType {
    Beauty,
    //ShadNorm,
    //GeomNorm,
    //Variance,
    //DirectLight,
    //...
}

pub const TILE_DIM: usize = 8;
pub const TILE_LEN: usize = TILE_DIM * TILE_DIM;

// A dynamic pixel buffer (dynamic in the sense that it can be
// used to indicate how to access pixel buffers)
pub struct PixelBuffer {
    type_id: PixelType,
    byte_buf: Vec<u8>,
    num_tiles: usize,
}

impl PixelBuffer {
    pub fn new<P: Pixel>(num_tiles: usize, init: P) -> Self {
        let buf = vec![[init; TILE_LEN]; num_tiles];
        PixelBuffer {
            type_id: P::TypeID,
            byte_buf: unsafe { memory::transmute_vec(buf) },
            num_tiles,
        }
    }

    // Instead of having a set and get function, just always return a mutable reference.
    // It's unsafe because it borrows the buffer immutably, but returns a mutable ref.
    pub unsafe fn get_tile<P: Pixel>(&self, index: usize) -> &mut [P; TILE_LEN] {
        let tile_size = std::mem::size_of::<[P; TILE_LEN]>();
        let byte_index = index * tile_size;
        let ptr = &self.byte_buf[byte_index] as *const u8;
        &mut *std::mem::transmute_copy::<_, *mut [P; TILE_LEN]>(&ptr)
    }
}
