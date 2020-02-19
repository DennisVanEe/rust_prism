use crate::filter::PixelFilter;
use crate::math::vector::Vec2;
use crate::spectrum::{Spectrum, XYZColor};
use crate::math::util;

use std::sync::atomic::{Ordering, AtomicUsize};
use std::iter::IntoIterator;
use std::slice::Iter;
use std::mem;

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
        let pos = util::morton_to_2d(index as u64);
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
        util::morton_from_2d(pos) as usize
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
pub struct PixelTile<P: Pixel> {
    pub data: [P; TILE_DIM * TILE_DIM], // The actual data that we care about
    // A lot of this information is redundant (can all be computed if given
    // the original pixel buffer). But if a thread doesn't want to deal with
    // checking the PixelBuffer, all of the info is here:
    tile_index: usize,      // The index of the tile in question
    tile_pos: Vec2<usize>,  // The (x, y) position of the tile itself
    pixel_pos: Vec2<usize>, // The positin of the top left pixel
}

impl<P: Pixel> PixelTile<P> {
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

pub struct PixelTileIter<'a, P: Pixel> {
    tile_iter: Iter<'a, P>,
    pixel_pos: Vec2<usize>, // Useful when sampling
}

impl<'a, P: Pixel> Iterator for PixelTileIter<'a, P> {
    type Item = (P, Vec2<usize>); // The global position on film and the pixel

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pixel) = self.tile_iter.next() {
            let result = (pixel, self.pixel_pos);
            // update pixel position:
            
        } else {
            None
        }
    }
}

//
// Film
// A collection (in the future) of pixel buffers that lends out tiles for
// processing and whatnot
//

// Because we are using importance sampling, the weights of each sample
// is 1. So we can just keep a count of the number of samples we have for that
// pixel. It's 32 bits (4294967296 samples is a lot of samples per pixel...)

#[derive(Clone, Copy)]
struct FilmPixel {
    pub value: XYZColor,
    pub count: u32,
}

impl Pixel for FilmPixel {
    type FinalOutput = Spectrum;

    fn zero() -> Self {
        FilmPixel {
            value: XYZColor::zero(),
            count: 0,
        }
    }

    fn update(&mut self, p: &Self) {
        // Relatively simple update function:
        self.value = self.value + p.value;
        self.count = self.count + p.count;
    }

    fn set_zero(&mut self) {
        self.value = XYZColor::zero();
        self.count = 0;
    }

    fn finalize(&self) -> Self::FinalOutput {
        // First we normalize the XYZColor value:
        let weight = 1. / (self.count as f64);
        let final_xyz = self.value.scale(weight);
        // Convert it to RGBColor space:
        Spectrum::from_xyz(final_xyz)
    }
}

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct Film<O: TileOrdering> {
    film_pixels: Vec<[FilmPixel; TILE_DIM * TILE_DIM]>,

    ordering: O,                  // The order in which we visit each tile
    filter: PixelFilter,          // The pixel filter used to determine how to sample pixels
    tile_res: Vec2<usize>,        // The resolution in terms of tiles
    pixel_res: Vec2<usize>,       // The resolution in terms of pixels
    curr_tile_index: AtomicUsize, // A simple atomic counter that counts to the max value of data
}

impl<O: TileOrdering> Film<O> {
    // Creates a new pixel buffer given the tile resolution.
    pub fn new_zero(tile_res: Vec2<usize>, filter: PixelFilter) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);

        let film_pixels = vec![[FilmPixel::zero(); TILE_DIM * TILE_DIM]; tile_res.x * tile_res.y];
        Film {
            film_pixels,
            ordering: O::new(tile_res),
            filter,
            tile_res,
            pixel_res,
            curr_tile_index: AtomicUsize::new(0),
        }
    }

    pub fn new(tile_res: Vec2<usize>, filter: PixelFilter, pixel: FilmPixel) -> Self {
        let pixel_res = tile_res.scale(TILE_DIM);

        let film_pixels = vec![[pixel; TILE_DIM * TILE_DIM]; tile_res.x * tile_res.y];
        Film {
            film_pixels,
            ordering: O::new(tile_res),
            filter,
            tile_res,
            pixel_res,
            curr_tile_index: AtomicUsize::new(0),
        }
    }

    // Sets the entire pixel buffer to the pixel's "zero" state
    pub fn set_zero(&mut self) {
        self.film_pixels.iter_mut().for_each(|tile| {
            tile.iter_mut().for_each(|p| {
                p.set_zero();
            });
        });
    }

    // Returns a zeroed tile for the given tile_index:
    pub fn get_zero_tile(&self) -> Option<FilmTile<FilmPixel>> {
        if let Some(tile_index) = self.get_next_tile_index() {
            let tile_pos = self.ordering.get_pos(tile_index);
            Some(FilmTile {
                data: [FilmPixel::zero(); TILE_DIM * TILE_DIM],
                tile_index,
                tile_pos,
                pixel_pos: tile_pos.scale(TILE_DIM),
            })
        } else {
            None
        }
    }

    // Returns the tile data present at the given tile_index:
    pub fn get_tile(&self) -> Option<PixelTile<FilmPixel>> {
        if let Some(tile_index) = self.get_next_tile_index() {
            let tile_pos = self.ordering.get_pos(tile_index);
            Some(PixelTile {
                data: self.data[tile_index],
                tile_index,
                tile_pos,
                pixel_pos: tile_pos.scale(TILE_DIM),
            })
        } else {
            None
        }
    }

    // Given a tile, updates the values in that location. Because of the way that
    // AtomicUsize is implemented, we can gaurantee that no two tiles will write
    // the same location:
    pub fn update_tile(&self, tile: &FilmTile<FilmPixel>) {
        // We have a gaurantee that this will be safe:
        let mut_self = unsafe { mem::transmute::<&Self, &mut Self>(self) };

        let buffer_tile = mut_self.data[tile.tile_index];
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

    fn get_next_tile_index(&self) -> Option<usize> {
        // Get the current tile we have:
        let mut old_tile = self.curr_tile_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.data.len() {
                // When I'm working on adding adaptive sampling, I can change what the tile index should
                //  be once I've gone through all possible options here:
                // 0
                return None;
            } else {
                old_tile + 1
            };

            if let Err(x) = self.curr_tile_index.compare_exchange_weak(old_tile, new_tile, Ordering::Relaxed, Ordering::Relaxed) {
                // Someone else changed the value, oh well, try again with a different x value:
                old_tile = x;
            } else {
                // We return the "old_tile". The new_tile is for the next time we run the code:
                return Some(old_tile);
            }
        }
    }
}
