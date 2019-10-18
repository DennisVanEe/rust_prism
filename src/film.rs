use crate::filter::{Filter, PixelFilter};
use crate::math::vector::Vec2;
use crate::pixel_buffer::{Pixel, PixelBuffer, PixelTile, TILE_DIM};
use crate::spectrum::XYZColor;

use simple_error::{bail, SimpleResult};

use std::mem::transmute;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Because we are using importance sampling, the weights of each sample
// is 1. So we can just keep a count of the number of samples we have for that
// pixel. It's 32 bits (4294967296 samples is a lot of samples per pixel...)

#[derive(Clone, Copy)]
struct FilmPixel {
    pub value: XYZColor,
    pub count: u32,
}

impl Pixel for FilmPixel {
    fn zero() -> Self {
        FilmPixel {
            value: XYZColor::zero(),
            count: 0,
        }
    }

    fn update(&mut self, p: &Self) {
        self.value = self.value + p.value;
        self.count = self.count + p.count;
    }

    fn set_zero(&mut self) {
        self.value = XYZColor::zero();
        self.count = 0;
    }
}

// A special type that can only be created by a Film object.
// This way, if the Film object works, we can gaurantee that
// no two threads will ever have a tile to the same location
// at the same time.
//
// A FilmTile is not copyable (even though it could be) and not
// clonable to prevent a thread from keeping a copy of the Film
// tile and submitting it later (which could cause problems). Why
// someone would write code like that does that is beyond me, though.
pub struct FilmTile {
    pub tile: PixelTile<FilmPixel>,
}

impl FilmTile {
    fn new(tile: PixelTile<FilmPixel>) -> Self {
        Self { tile }
    }
}

// The film class is in charge of managing all information about the film we may want.
pub struct Film {
    pixel_buffer: PixelBuffer<FilmPixel>,
    pixel_filter: PixelFilter,

    // Used for basic unfirom scanline sampling. Fancier, adaptive
    // sampling techniques will be explored in the future (TODO for good
    // measure!)
    next_tile: AtomicUsize,
    // In case we have multiple passes:
    is_done: AtomicBool,
}

impl Film {
    // This performs the check to make sure that the resolution
    // provided is a multiple of the TILE_DIM. I could remove this
    // constraint, but that would make the code a little bit more
    // complex, and I don't feel like doing that:
    pub fn new<T: Filter>(filter: &T, pixel_res: Vec2<usize>) -> SimpleResult<Self> {
        // First we check if the resolution is a multiple of
        // the TILE_DIM:
        if pixel_res.x % TILE_DIM != 0 || pixel_res.y % TILE_DIM != 0 {
            bail!(
                "The provided Film resolution must be a multiple of: {}",
                TILE_DIM
            );
        }
        let tile_res = Vec2 {
            x: pixel_res.x / TILE_DIM,
            y: pixel_res.y / TILE_DIM,
        };
        Ok(Film {
            pixel_buffer: PixelBuffer::new_zero(tile_res),
            pixel_filter: PixelFilter::new(filter),
            next_tile: AtomicUsize::new(0),
            is_done: AtomicBool::new(false),
        })
    }

    pub fn is_done(&self) -> bool {
        self.is_done.load(Ordering::Relaxed)
    }

    // Returns the next tile for a thread to work on. If no tiles are left to be worked on.
    pub fn next_tile(&self) -> Option<FilmTile> {
        // Check if we are done. I am aware that this state could change from here to the next instruction.
        // If that does happen, however, the code should still work.
        if self.is_done() {
            return None;
        }

        // We are doing a simple scan-line approach here, so we just increment the counter. We
        // are also currently using simple uniform sampling (no adaptive sampling). With adaptive
        // sampling this could potentially be a lot more difficult. Future me will worry about that.
        //
        // Now, because of wrapping behaviour, if this gets called MANY times after we are done,
        // it could potentially wrap around and falsly return true.
        let tile_index = self.next_tile.fetch_add(1, Ordering::Relaxed);
        // Check if this is a valid tile or not:
        if tile_index >= self.pixel_buffer.get_num_tiles() {
            // This may be called multiple times, but because they will all set it to true,
            // it shouldn't be a problem.
            self.is_done.store(true, Ordering::Relaxed);
            return None;
        }

        // Otherwise we can just create the tile:
        Some(FilmTile::new(self.pixel_buffer.get_zero_tile(tile_index)))
    }

    // Updates the film buffer at the specified location with the given tile. Technically
    // this is a mutable operation, but because of the use of FilmTiles, we can gaurantee
    // that everyone accessing it will access a disjoint portion of it:
    pub fn update_tile(&self, film_tile: FilmTile) {
        // We can do this for the reason described above:
        let mut_self = unsafe { transmute::<&Self, &mut Self>(self) };
        // Now we can just go ahead and update the tile:
        mut_self.pixel_buffer.update_tile(&film_tile.tile);
    }

    pub fn get_pixel_res(&self) -> Vec2<usize> {
        self.pixel_buffer.get_pixel_res()
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.pixel_buffer.get_tile_res()
    }
}