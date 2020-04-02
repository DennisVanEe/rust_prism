use crate::math::util;
use crate::math::vector::Vec2;
use crate::spectrum::{Spectrum, XYZColor};

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A pixel is NOT an AOV. An AOV is a value that can be written to
/// but doesn't have to be. A Pixel must always be present. Any information that
/// is necessary goes here:
#[derive(Clone, Copy)]
pub struct Pixel {
    pub color: XYZColor,
    // Maybe?: variance: f64,
}

impl Pixel {
    /// Creates an instance of a pixel in the zero state.
    pub fn zero() -> Self {
        Pixel {
            color: XYZColor::zero(),
        }
    }

    pub fn from_render_pixel(rp: RenderPixel) -> Self {
        Pixel {
            color: rp.color.to_xyz(),
        }
    }

    pub fn to_render_pixel(self) -> RenderPixel {
        RenderPixel {
            color: Spectrum::from_xyz(self.color),
        }
    }
}

/// This is the value of the pixel when its given to the integrator.
/// This is not necessarily different from that of the Pixel.
#[derive(Clone, Copy)]
pub struct RenderPixel {
    pub color: Spectrum,
    // Maybe?: variance: f64
}

pub const TILE_DIM: usize = 8;
pub const TILE_LEN: usize = TILE_DIM * TILE_DIM;

/// A FilmTile holds all of the information that a rendering thread needs from
/// the film buffer.
pub struct FilmTile<'a> {
    /// A reference to the tile data
    pub data: [Pixel; TILE_LEN],
    /// The coordinate of the top left most pixel in the tile.
    pub pos: Vec2<usize>,
    /// A unique seed for use with the samplers. Even if it's technically the same
    /// tile, the seed will always be unique.
    pub seed: u64,
    // The index of the tile in the buffer.
    index: usize,
}

/// Manages the pixel buffer and the tile scheduler. For simple cases, the tile scheduler just moves
/// through the tiles in a linear fashion. But when adaptive sampling is implemented, these operations
/// will become more complex. Because it's in charge of adaptive sampling, the Film object is in charge
/// of ending the rendering process when it deems enough tiles to have been rendered.
pub struct Film {
    buffer: Vec<Cell<[Pixel; TILE_LEN]>>, // The buffer that stores the tiles.
    tile_res: Vec2<usize>,                // The resolution in terms of tiles.
    next_tile_index: AtomicUsize,         // The next tile to "hand out".
}

impl Film {
    /// Generates a new Film struct.
    ///
    /// # Arguments
    /// * `tile_res` - The resolution, in tiles, of the Film
    /// * `init_pixel` - What value every pixel in the Film should initially have
    ///
    /// # Panics
    /// If `tile_res` leads to the total number of tiles being zero, new will panic.
    pub fn new(tile_res: Vec2<usize>, init_pixel: Pixel) -> Self {
        let num_tiles = tile_res.x * tile_res.y;
        assert_ne!(num_tiles, 0);
        Film {
            buffer: vec![[init_pixel; TILE_LEN]; num_tiles],
            tile_res,
            next_tile_index: AtomicUsize::new(0),
        }
    }

    /// Sets every pixel in the Film struct to zero.
    pub fn set_zero(&mut self) {
        for tile in self.buffer.iter_mut() {
            for pixel in tile.iter_mut() {
                pixel.set(Pixel::zero());
            }
        }
    }

    /// A thread safe function that returns a tile for a single thread to work with.
    /// If the function returns `None`, then we have finished rendering.
    pub fn get_tile(&self) -> Option<FilmTile> {
        let mut old_tile = self.next_tile_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.buffer.len() {
                return None;
            } else {
                old_tile + 1
            };

            if let Err(i) = self.cur_index.compare_exchange_weak(
                old_tile,
                new_tile,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                // Someone else changed the value, oh well, try again with a different i value:
                old_tile = i;
            } else {
                // We have specified a tile now:
                break;
            }
        }

        let pos_u32 = util::morton_to_2d(old_tile as u64);
        return Some(FilmTile {
            data: self.buffer[old_tile].get(),
            pos: Vec2 {
                x: pos_u32.x as usize,
                y: pos_u32.y as usize,
            },
            // We aren't doing anything fancy yet, so each tile gets hit once.
            seed: old_tile as u64,
        });
    }

    /// Updates the buffer with the current tile with a given film tile.
    ///
    /// # Arguments
    /// * `tile` - The tile value that was rendered to that is being updated.
    pub fn set_tile(&self, tile: FilmTile) {
        self.buffer[tile.index].set(tile.data);
    }

    /// Returns the current progress in terms of a percentage.
    pub fn get_percent_complete(&self) -> f64 {
        let num_tiles = self.buffer.len() as f64;
        let done = self.next_tile_index.load(Ordering::Relaxed) as f64;
        done / num_tiles
    }
}
