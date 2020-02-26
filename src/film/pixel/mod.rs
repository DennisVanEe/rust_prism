use crate::memory;

use enum_map::Enum;

pub mod beauty;

trait Pixel: Copy {
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
pub (super) enum PixelType {
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
pub (super) struct PixelBuffer {
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

    // TODO: Comlete this implementation here:
}
