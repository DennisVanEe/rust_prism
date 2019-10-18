// Can be used throughout to store information in a "PixelBuffer"
// like manner.

use crate::math::vector::Vec2;

//
// Pixel
//

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

//
// PixelTile
//

// Now, a single thread renderes a collection of pixels (not just one).
// It renders in tiles which are of a certain size as defined here:
// TILE_DIM means a tile is TILE_DIM X TILE_DIM:
pub const TILE_DIM: usize = 8;

// This is the tile that is given to a thread for it to fill out with important
// information.
pub struct PixelTile<T: Pixel> {
    pub data: [T; TILE_DIM * TILE_DIM], // The actual data that we care about
    // A lot of this information is redundant (can all be computed if given
    // the original pixel buffer). But if a thread doesn't want to deal with
    // checking the PixelBuffer, all of the info is here:
    tile_index: usize,      // The index of the tile in question
    tile_vec: Vec2<usize>,  // The (x, y) position of the tile itself
    pixel_vec: Vec2<usize>, // The positin of the top left pixel
}

impl<T: Pixel> PixelTile<T> {
    pub fn get_tile_index(&self) -> usize {
        self.tile_index
    }

    pub fn get_tile_vec(&self) -> Vec2<usize> {
        self.tile_vec
    }

    pub fn get_pixel_vec(&self) -> Vec2<usize> {
        self.pixel_vec
    }
}

//
// PixelBuffer
//

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

    pub fn tile_index_to_vec(&self, tile_index: usize) -> Vec2<usize> {
        Vec2 {
            x: tile_index % self.tile_res.x,
            y: tile_index / self.tile_res.x,
        }
    }

    pub fn tile_vec_to_index(&self, tile_vec: Vec2<usize>) -> usize {
        self.tile_res.x * tile_vec.y + tile_vec.x
    }

    // Returns a zeroed tile for the given tile_index:
    pub fn get_zero_tile(&self, tile_index: usize) -> PixelTile<T> {
        let tile_vec = self.tile_index_to_vec(tile_index);
        PixelTile {
            data: [T::zero(); TILE_DIM * TILE_DIM],
            tile_index,
            tile_vec,
            pixel_vec: tile_vec.scale(TILE_DIM),
        }
    }

    // Returns the tile data present at the given tile_index:
    pub fn get_tile(&self, tile_index: usize) -> PixelTile<T> {
        let tile_vec = self.tile_index_to_vec(tile_index);
        PixelTile {
            data: self.data[tile_index],
            tile_index,
            tile_vec,
            pixel_vec: tile_vec.scale(TILE_DIM),
        }
    }

    // Given a tile, updates the values in that location:
    pub fn update_tile(&mut self, tile: &PixelTile<T>) {
        let buffer_tile = &mut self.data[tile.tile_index];
        buffer_tile
            .iter_mut()
            .zip(tile.data.iter())
            .for_each(|(curr_p, p)| {
                curr_p.update(p);
            });
    }

    pub fn get_num_tiles(&self) -> usize {
        self.data.len()
    }

    pub fn get_pixel_res(&self) -> Vec2<usize> {
        self.pixel_res
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.tile_res
    }
}
