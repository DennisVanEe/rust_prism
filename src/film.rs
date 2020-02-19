use crate::filter::{Filter, PixelFilter};
use crate::math::vector::Vec2;
use crate::pixel_buffer::{Pixel, PixelBuffer, PixelTile, TileOrdering, TILE_DIM};
use crate::spectrum::{Spectrum, XYZColor};

use simple_error::{bail, SimpleResult};

use std::mem::transmute;
use std::sync::atomic::{Ordering, AtomicUsize};

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

// A special type that can only be created by a Film object.
// This way, if the Film object works, we can gaurantee that
// no two threads will ever have a tile to the same location
// at the same time.
//
// A FilmTile is not copyable (even though it could be) and not
// clonable to prevent a thread from keeping a copy of the Film
// tile and submitting it later (which could cause problems).
pub struct FilmTile {
    pub tile: PixelTile<FilmPixel>,
}

impl FilmTile {
    fn new(tile: PixelTile<FilmPixel>) -> Self {
        Self { tile }
    }
}

// The film class is in charge of managing all information about the film we may want.
// TODO: the pixel buffer will change for some sort of adaptive sampling pixel buffer in the future,
// such a buffer would allow me to define when to "end" the operation
pub struct Film<O: TileOrdering> {
    pixel_buffer: PixelBuffer<FilmPixel, O>,
    pixel_filter: PixelFilter,
    curr_tile_index: AtomicUsize,
}

impl<O: TileOrdering> Film<O> {
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
            curr_tile_index: AtomicUsize::new(0),
        })
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

    // Returns None when all tiles are complete:
    pub fn next_tile(&self) -> Option<FilmTile> {
        let tile_index = self.curr_tile_index.fetch_add(1, Ordering::Relaxed);
        
        if let Some(tile) = self.pixel_buffer.get_zero_tile(tile_index) {
            Some(FilmTile::new(tile))
        } else {
            None
        }
    }

    pub fn get_pixel_res(&self) -> Vec2<usize> {
        self.pixel_buffer.get_pixel_res()
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.pixel_buffer.get_tile_res()
    }
}
