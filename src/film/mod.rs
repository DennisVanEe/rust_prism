use crate::math::vector::Vec2;

use pixel::{Pixel, PixelBuffer, PixelType, TILE_LEN};

pub mod pixel;
pub mod tile_schedular;

use enum_map::EnumMap;

// Used to index a specific tile from the film:
pub struct TileIndex {
    // Linear index of the struct:
    index: usize,
    // pixel position of the tile (top left):
    pixel_pos: Vec2<usize>,
}

impl TileIndex {
    pub fn pixel_pos(&self) -> Vec2<usize> {
        self.pixel_pos
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

    pub fn get_tile<P: Pixel>(&self, index: TileIndex) -> Option<&mut [P; TILE_LEN]> {
        if let Some(buff) = &self.pixel_buffs[P::TypeID] {
            Some(unsafe { buff.get_tile::<P>(index.index) })
        } else {
            None
        }
    }
}
