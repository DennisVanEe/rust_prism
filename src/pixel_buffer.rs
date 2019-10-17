// Can be used throughout to store information in a "PixelBuffer"
// like manner.

use crate::math::vector::Vec2;

// This trait is needed so we know how to update a pixel
// when a PixelBuffer is given a tile that we recently finished
// work on:
pub trait Pixel: Copy {
    // Create an instance of the pixel in the "zero" state:
    fn zero() -> Self;
    // Sets the buffer to zero:
    fn set_zero(&mut self);
    // Update the current pixel given another pixel:
    fn update(&mut self, p: &Self);
}

// Now, a single thread renderes a collection of pixels (not just one).
// It renders in tiles which are of a certain size as defined here:
// TILE_DIM means a tile is TILE_DIM X TILE_DIM:
pub const TILE_DIM: usize = 8;

// This is the tile that is given to a thread for it to fill out with important
// information.
pub struct PixelTile<T: Pixel> {
    pub data: [T; TILE_DIM * TILE_DIM], // The actual data that we care about
    pub tile_pos: Vec2<usize>,          // The (x, y) position of the tile itself
    pub pixel_pos: Vec2<usize>,         // The positin of the top left pixel
}

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct PixelBuffer<T: Pixel> {
    // The data is stored in order of morton curves:
    data: Vec<[T; TILE_DIM * TILE_DIM]>,
    tile_res: Vec2<usize>,
    pixel_res: Vec2<usize>,
}

impl<T: Pixel> PixelBuffer<T> {
    // Creates a new pixel buffer given the tile resolution.
    pub fn new_zero(tile_res: Vec2<usize>) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);

        let data = vec![[T::zero(); TILE_DIM * TILE_DIM]; tile_res.x * tile_res.y];
        PixelBuffer {
            data,
            tile_res,
            pixel_res,
        }
    }

    pub fn new(tile_res: Vec2<usize>, pixel: T) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);

        let data = vec![[pixel; TILE_DIM * TILE_DIM]; tile_res.x * tile_res.y];
        PixelBuffer {
            data,
            tile_res,
            pixel_res,
        }
    }

    // Sets the entire pixel buffer to the pixel's "zero" state
    pub fn set_zero(&mut self) {
        self.data.iter_mut().for_each(|tile| {
            tile.iter_mut().for_each(|p| {
                p.set_zero();
            });
        });
    }

    // Given a tile, updates the values in that location:
    pub fn update(&mut self, tile: &PixelTile<T>) {
        // The specific tile we are interested:
        let tile_index = tile.tile_pos.x + tile.tile_pos.y * self.tile_res.x;
        // Measrable performance improvement:
        let buffer_tile = unsafe { self.data.get_unchecked_mut(tile_index) };

        buffer_tile.iter_mut()
            .zip(tile.data.iter())
            .for_each(|(curr_p, p)| {
                curr_p.update(p);
            });
    }

    pub fn get_pixel_res(&self) -> Vec2<usize> {
        self.pixel_res
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.tile_res
    }
}
