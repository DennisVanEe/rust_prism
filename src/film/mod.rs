use crate::math::vector::Vec2;

use pixel::{Pixel, PixelBuffer, PixelType, TILE_DIM, TILE_LEN};

pub mod pixel;
pub mod tile_schedular;

use enum_map::EnumMap;

#[derive(Clone, Copy)]
pub struct TileIndex {
    index: usize,       // The linear index of the tile into film
    seed: u64,          // Used to specify a unique seed between different tile iterations. Used for the sampler
    pos: Vec2<usize>,   // The pixel position of the top left corner of a tile
}

impl TileIndex {
    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn pos(&self) -> Vec2<usize> {
        self.pos
    }
}

pub struct Film {
    pixel_buffs: EnumMap<PixelType, Option<PixelBuffer>>,
    // Some properties about the film itself:
    tile_res: Vec2<usize>,
    pixel_res: Vec2<usize>,
    num_tiles: usize,
}

impl Film {
    pub fn new(tile_res: Vec2<usize>) -> Self {
        Film {
            pixel_buffs: EnumMap::new(),
            tile_res: tile_res,
            pixel_res: tile_res.scale(pixel::TILE_DIM),
            num_tiles: tile_res.x * tile_res.y,
        }
    }

    // Adds a new buffer to the film. Doesn't have to be thread safe:
    pub fn add_buff<P: Pixel>(&mut self, init: P) {
        self.pixel_buffs[P::TypeID] = Some(PixelBuffer::new::<P>(self.num_tiles, init));
    }

    // Returns a tile given a tile index:
    pub fn get_tile<P: Pixel>(&self, index: TileIndex) -> Option<&mut [P; TILE_LEN]> {
        if let Some(buff) = &self.pixel_buffs[P::TypeID] {
            Some(unsafe { buff.get_tile::<P>(index.index) })
        } else {
            None
        }
    }
}

// This is what actually gets passed to the integrator:
pub struct FilmPixel<'a> {
    film: &'a Film,
    tile_index: TileIndex,
    pixel_index: usize,   
}

impl<'a> FilmPixel<'a> {
    pub fn new(film: &Film, tile_index: TileIndex) -> Self {
        FilmPixel {
            film,
            tile_index,
            pixel_index: 0,
        }
    }

    // Returns the position of the pixel in (x, y) coordinates (top left corner):
    pub fn pos(&self) -> Vec2<usize> {
        let delta = Vec2 {
            x: self.pixel_index % TILE_DIM,
            y: self.pixel_index / TILE_DIM,
        };
        self.tile_index.pos() + delta
    }

    pub fn get<P: Pixel>(&self) -> Option<&mut P> {
        if let Some(tile) = self.film.get_tile::<P>(&self.tile_index) {
            Some(&mut tile[self.pixel_index])
        } else {
            None
        }
    }

    // Retrieves the next pixel until we need a new tile:
    pub fn next_pixel(&mut self) -> bool {
        if self.pixel_index == TILE_LEN {
            false
        } else {
            self.pixel_index += 1;
            true
        }
    }

    pub fn set_tile(&mut self, tile: TileIndex) {
        self.tile_index = tile;
        self.pixel_index = 0;
    }
}
