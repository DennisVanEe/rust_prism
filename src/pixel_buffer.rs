// Can be used throughout to store information in a "PixelBuffer"
// like manner.

use crate::math::util::morton_from_2d;
use crate::math::vector::Vec2;

// This trait is needed so we know how to update a pixel
// when a PixelBuffer is given a tile that we recently finished
// work on:
pub trait Pixel: Copy {
    // Create an instance of itself that is in a "zero" state:
    fn zero() -> Self;
    // Sets the buffer to zero:
    fn set_zero(&mut self);
    // Update a pixel given another pixel:
    fn update(&mut self, p: &Self);
}

// Now, a single thread renderes a collection of pixels (not just one).
// It renders in tiles which are of a certain size as defined here:
// TILE_DIM means a tile is TILE_DIM X TILE_DIM:
pub const TILE_DIM: usize = 8;

// An individual tile. This is what a single thread
// will actually be working on:
pub struct PixelTile<T: Pixel> {
    pub data: [T; TILE_DIM * TILE_DIM], // The actual data that we care about
    tile_pos: Vec2<u32>,                // The position in terms of pixels of this tile
}

impl<T: Pixel> PixelTile<T> {
    pub fn get_tile_pos(&self) -> Vec2<u32> {
        self.tile_pos
    }

    pub fn set_zero(&mut self) {
        for p in self.data.iter_mut() {
            p.set_zero();
        }
    }

    fn update(&mut self, tile: &PixelTile<T>) {
        // Hopefully this unrolls this properly as data is a statically sized array:
        self.data
            .iter_mut()
            .zip(tile.data.iter())
            .for_each(|(curr_p, update_p)| {
                curr_p.update(update_p);
            });
    }
}

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct PixelBuffer<T: Pixel> {
    // The data is stored in order of morton curves:
    data: Vec<PixelTile<T>>,
    tile_res: Vec2<usize>,
    pixel_res: Vec2<usize>,
}

impl<T: Pixel> PixelBuffer<T> {
    // Creates a new pixel buffer given the tile resolution.
    pub fn new(tile_res: Vec2<usize>) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);
        let total_size = pixel_res.x * pixel_res.y;
        // Make sure to set the entire buffer to zero:
        let data = vec![T::zero(); total_size];
        PixelBuffer {
            data,
            tile_res,
            pixel_res,
        }
    }

    // Sets the entire pixel buffer to the pixel's "zero" state
    pub fn set_zero(&mut self) {
        self.data.iter_mut().for_each(|p| {
            p.set_zero();
        });
    }

    // Given a tile, updates the values in that location:
    pub fn update_tile(&mut self, tile: &PixelTile<T>) {
        // The specific tile we are interested:
        let data_tile_index = morton_from_2d(tile.tile_pos) as usize;

        let pixel_tile_index = data_tile_index * (TILE_DIM * TILE_DIM);
        let data = &mut self.data[pixel_tile_index..(pixel_tile_index + TILE_DIM * TILE_DIM)];
        data.iter_mut()
            .zip(tile.data.iter())
            .for_each(|(curr_p, update_p)| {
                curr_p.update(update_p);
            });
    }

    pub fn get_pixel_res(&self) -> Vec2<usize> {
        self.pixel_res
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.tile_res
    }
}
