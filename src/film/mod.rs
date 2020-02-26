use crate::math::vector::Vec2;

use pixel::{PixelBuffer, PixelType};

pub mod tile_schedular;
pub mod pixel;

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
        self.aov_pos
    }
}

pub struct Film {
    pixel_buffs: EnumMap<PixelType, PixelBuffer>,
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

    // TODO: add functions for adding buffers, reseting buffers,
    // and adding tiles and whatnot
}
