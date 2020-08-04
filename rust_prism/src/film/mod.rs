use crate::spectrum::Color;
use pmath::vector::Vec2;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod png;

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub color: Color,
    pub count: u32,
}

impl Pixel {
    /// Creates an instance of a pixel that is black.
    pub fn black() -> Self {
        Pixel {
            color: Color::black(),
            count: 0,
        }
    }

    /// Creates an instance of a pixel that is white.
    pub fn white() -> Self {
        Pixel {
            color: Color::white(),
            count: 0,
        }
    }

    /// Creates a new pixel with the given spectrum.
    pub fn new(color: Color) -> Self {
        Pixel { color, count: 0 }
    }

    /// Adds a sample to the pixel.
    pub fn add_sample(self, color: Color) -> Self {
        Pixel {
            color: self.color + color,
            count: self.count + 1,
        }
    }

    /// Calculates the final color of the pixel.
    pub fn final_color(self) -> Color {
        if self.count == 0 {
            self.color
        } else {
            self.color.scale(1.0 / (self.count as f64))
        }
    }
}

pub const TILE_DIM: usize = 16;
pub const TILE_SIZE: usize = TILE_DIM * TILE_DIM;

/// Given an index, uniquely maps it to a 2d position.
fn index_to_pos(index: u64, res: Vec2<usize>) -> Vec2<u32> {
    // Simple scanline for now:
    Vec2 {
        x: (index % (res.x as u64)) as u32,
        y: (index / (res.x as u64)) as u32,
    }
}

// A FilmTile holds all of the information that a rendering thread needs from
// the film buffer.
pub struct FilmTile {
    // The data in a specific tile.
    pub data: [Pixel; TILE_SIZE],
    // The coordinate of the top left most pixel in the tile.
    pub pos: Vec2<usize>,
    // A unique seed for use with the samplers. Even if it's technically the same
    // tile, the seed will always be unique.
    pub seed: u64,
    // The index of the tile in the buffer.
    pub index: usize,
}

// Manages the pixel buffer and the tile scheduler. For simple cases, the tile scheduler just moves
// through the tiles in a linear fashion. But when adaptive sampling is implemented, these operations
// will become more complex. Because it's in charge of adaptive sampling, the Film object is in charge
// of ending the rendering process when it deems enough tiles to have been rendered.
pub struct Film {
    buffer: Vec<Cell<[Pixel; TILE_SIZE]>>, // The buffer that stores the tiles.
    tile_res: Vec2<usize>,                 // The resolution in terms of tiles.
    next_tile_index: AtomicUsize,          // The next tile to "hand out".
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
    pub fn new(tile_res: Vec2<usize>, pixel: Pixel) -> Self {
        let num_tiles = tile_res.x * tile_res.y;
        //assert_ne!(num_tiles, 0);
        Film {
            buffer: vec![Cell::new([pixel; TILE_SIZE]); num_tiles],
            tile_res,
            next_tile_index: AtomicUsize::new(0),
        }
    }

    pub fn new_zero(tile_res: Vec2<usize>) -> Self {
        let num_tiles = tile_res.x * tile_res.y;
        //assert_ne!(num_tiles, 0);
        Film {
            buffer: vec![Cell::new([Pixel::black(); TILE_SIZE]); num_tiles],
            tile_res,
            next_tile_index: AtomicUsize::new(0),
        }
    }

    /// Sets every pixel in the Film struct to zero.
    pub fn reset(&mut self) {
        for tile in self.buffer.iter_mut() {
            for pixel in tile.get().iter_mut() {
                *pixel = Pixel::black();
            }
        }
    }

    // A thread safe function that returns a tile for a single thread to work with.
    // If the function returns `None`, then we have finished rendering.
    pub fn get_tile(&self) -> Option<FilmTile> {
        let mut old_tile = self.next_tile_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.buffer.len() {
                return None;
            } else {
                old_tile + 1
            };

            if let Err(i) = self.next_tile_index.compare_exchange_weak(
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

        let pos_u32 = index_to_pos(old_tile as u64, self.tile_res);
        return Some(FilmTile {
            data: self.buffer[old_tile].get(),
            pos: Vec2 {
                x: pos_u32.x as usize,
                y: pos_u32.y as usize,
            }
            .scale(TILE_DIM),
            // We aren't doing anything fancy yet, so each tile gets hit once.
            seed: old_tile as u64,
            index: old_tile,
        });
    }

    /// Updates the buffer with the current tile with a given film tile.
    pub fn set_tile(&self, tile: FilmTile) {
        self.buffer[tile.index].set(tile.data);
    }

    /// Returns the current progress in terms of a percentage.
    pub fn get_percent_complete(&self) -> f64 {
        let num_tiles = self.buffer.len() as f64;
        let done = self.next_tile_index.load(Ordering::Relaxed) as f64;
        done / num_tiles
    }

    /// Given a function that converts XYZColor to an rgb value (in the form of an ImageBuffer),
    /// returns an ImageBuffer.
    pub fn to_image_buffer(&self, transf: fn(Color) -> ImagePixel) -> ImageBuffer {
        let res = self.tile_res.scale(TILE_DIM);
        let mut buffer = vec![ImagePixel::zero(); res.x * res.y];

        // This doesn't have to be a particularly fast function, so it isn't.

        for (i, tile) in self.buffer.iter().enumerate() {
            let tile = tile.get();
            let tile_pos = index_to_pos(i as u64, self.tile_res);
            let pixel_corner = Vec2 {
                x: tile_pos.x as usize,
                y: tile_pos.y as usize,
            }
            .scale(TILE_DIM);
            let mut pixel_pos = pixel_corner;

            for (i, pixel) in tile.iter().enumerate() {
                let pixel_index = pixel_pos.y * res.x + pixel_pos.x;
                let final_color = pixel.final_color();
                //println!("{:?}", final_color);
                buffer[pixel_index] = transf(final_color);
                if (i + 1) % TILE_DIM == 0 {
                    pixel_pos.y += 1;
                    pixel_pos.x = pixel_corner.x;
                } else {
                    pixel_pos.x += 1;
                }
            }
        }

        ImageBuffer { buffer, res }
    }
}

// Cell doesn't implement Sync. But, the way each tile is accessed means there shouldn't
// be any race-conditions for the same Cell.
unsafe impl Sync for Film {}

//
// The image buffer is an intermediate type that the pixel buffer converts to so that we can
// easily convert this to an actual image format later.
//

#[derive(Clone, Copy, Debug)]
pub struct ImagePixel {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl ImagePixel {
    pub fn zero() -> Self {
        ImagePixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct ImageBuffer {
    /// This is in row-major format
    buffer: Vec<ImagePixel>,
    res: Vec2<usize>,
}
