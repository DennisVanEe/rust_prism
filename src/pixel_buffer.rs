// Can be used throughout to store information in a "PixelBuffer"
// like manner.

use crate::math::util::{morton_from_2d, morton_to_2d};
use crate::math::vector::Vec2;

use std::sync::atomic::{Ordering, AtomicUsize};

//
// TileOrdering
//

// Defines how the tiles in a pixel buffer should be ordered.
pub trait TileOrdering {
    // Sets the resolution. Some ordering schemes might take advantage of
    // this, others might not.
    fn new(res: Vec2<usize>) -> Self;
    // Converts a linear index to a position:
    fn get_pos(&self, index: usize) -> Vec2<usize>;
    // Converts a position to an index:
    fn get_index(&self, pos: Vec2<usize>) -> usize;
}

// Tile ordering that follows the morton code:
pub struct MortonOrder {}

impl TileOrdering for MortonOrder {
    fn new(_: Vec2<usize>) -> Self {
        MortonOrder {}
    }

    fn get_pos(&self, index: usize) -> Vec2<usize> {
        let pos = morton_to_2d(index as u64);
        Vec2 {
            x: pos.x as usize,
            y: pos.y as usize,
        }
    }

    fn get_index(&self, pos: Vec2<usize>) -> usize {
        let pos = Vec2 {
            x: pos.x as u32,
            y: pos.y as u32,
        };
        morton_from_2d(pos) as usize
    }
}

pub struct ScanlineOrder {
    res: Vec2<usize>,
}

impl TileOrdering for ScanlineOrder {
    fn new(res: Vec2<usize>) -> Self {
        ScanlineOrder { res }
    }

    fn get_pos(&self, index: usize) -> Vec2<usize> {
        Vec2 {
            x: index % self.res.x,
            y: index / self.res.x,
        }
    }

    fn get_index(&self, pos: Vec2<usize>) -> usize {
        self.res.x * pos.y + pos.x
    }
}

//
// Pixel
//

// This trait is needed so we know how to update a pixel
// when a PixelBuffer is given a tile that we recently finished
// work on:
pub trait Pixel: Copy {
    // The final output type of the image. After we run a
    // "finalize" function over the buffer this is the final
    // result of the buffer. Usually this can be the same type
    // as the pixel:
    type FinalOutput: Copy;

    // Create an instance of the pixel in the "zero" state:
    fn zero() -> Self;
    // Sets the buffer to zero:
    fn set_zero(&mut self);
    // Update the current pixel given another pixel:
    fn update(&mut self, p: &Self);
    // Outputs a "final" result for the current pixel:
    fn finalize(&self) -> Self::FinalOutput;
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
    tile_pos: Vec2<usize>,  // The (x, y) position of the tile itself
    pixel_pos: Vec2<usize>, // The positin of the top left pixel
}

impl<T: Pixel> PixelTile<T> {
    pub fn get_tile_index(&self) -> usize {
        self.tile_index
    }

    pub fn get_tile_pos(&self) -> Vec2<usize> {
        self.tile_pos
    }

    pub fn get_pixel_pos(&self) -> Vec2<usize> {
        self.pixel_pos
    }
}

//
// PixelBuffer
//

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct PixelBuffer<P: Pixel, O: TileOrdering> {
    // The data is stored in order of morton curves:
    data: Vec<[P; TILE_DIM * TILE_DIM]>,
    ordering: O,
    tile_res: Vec2<usize>,
    pixel_res: Vec2<usize>,
    curr_tile_index: AtomicUsize,
}

impl<P: Pixel, O: TileOrdering> PixelBuffer<P, O> {
    // Creates a new pixel buffer given the tile resolution.
    pub fn new_zero(tile_res: Vec2<usize>) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);

        let data = vec![[P::zero(); TILE_DIM * TILE_DIM]; tile_res.x * tile_res.y];
        PixelBuffer {
            data,
            ordering: O::new(tile_res),
            tile_res,
            pixel_res,
        }
    }

    pub fn new(tile_res: Vec2<usize>, pixel: P) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);

        let data = vec![[pixel; TILE_DIM * TILE_DIM]; tile_res.x * tile_res.y];
        PixelBuffer {
            data,
            ordering: O::new(tile_res),
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

    // Returns a zeroed tile for the given tile_index:
    pub fn get_zero_tile(&self, tile_index: usize) -> Option<PixelTile<P>> {
        if tile_index < self.data.len() {
            return None;
        }

        let tile_pos = self.ordering.get_pos(tile_index);
        Some(PixelTile {
            data: [P::zero(); TILE_DIM * TILE_DIM],
            tile_index,
            tile_pos,
            pixel_pos: tile_pos.scale(TILE_DIM),
        })
    }

    // Returns the tile data present at the given tile_index:
    pub fn get_tile(&self, tile_index: usize) -> Option<PixelTile<P>> {
        if tile_index < self.data.len() {
            return None;
        }

        let tile_pos = self.ordering.get_pos(tile_index);
        Some(PixelTile {
            data: self.data[tile_index],
            tile_index,
            tile_pos,
            pixel_pos: tile_pos.scale(TILE_DIM),
        })
    }

    // Given a tile, updates the values in that location:
    pub fn update_tile(&mut self, tile: &PixelTile<P>) {
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

    fn get_next_tile_index(&self) -> usize {
        let curr_tile = 
    }
}
