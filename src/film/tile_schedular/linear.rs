use crate::math::util;
use crate::math::vector::Vec2;

use super::super::pixel::{TILE_DIM, TILE_LEN};
use super::super::TileIndex;
use super::TileSchedular;

use std::sync::atomic::{AtomicUsize, Ordering};

// A very basic linear tile schedular that just traverses the entire tile:
pub struct LinearTileSchedular {
    num_tiles: usize,
    cur_index: AtomicUsize,
}

impl TileSchedular for LinearTileSchedular {
    // The number of times to traverse the tiles:
    type Param = ();

    fn new(tile_res: Vec2<usize>, _: ()) -> Self {
        let num_tiles = tile_res.x * tile_res.y;
        LinearTileSchedular {
            num_tiles,
            cur_index: AtomicUsize::new(0),
        }
    }

    fn reset(&mut self) {
        self.cur_index.store(0, Ordering::Relaxed);
    }

    fn init_index(&mut self) -> Option<TileIndex> {
        // Because it doesn't have to be thread safe, we can do the
        // more naive approach here:
        let index = self.cur_index.load(Ordering::Relaxed);
        if index < self.num_tiles {
            // Let them out in morton order:
            self.cur_index.store(index + 1, Ordering::Relaxed);
            let pos_u32 = util::morton_to_2d(index as u64);
            let pos = Vec2 {
                x: pos_u32.x as usize,
                y: pos_u32.y as usize,
            };
            Some(TileIndex {
                index,
                seed: index as u64, // Because we only visit each tile once, this should work
                pos,
            })
        } else {
            None
        }
    }

    fn next_index(&self, mut index: TileIndex) -> Option<TileIndex> {
        // Get the current tile we have:
        let mut old_tile = self.cur_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.num_tiles {
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
                // We return the "old_tile". The new_tile is for the next time we run the code:
                let pos_u32 = util::morton_to_2d(old_tile as u64);
                return Some(TileIndex {
                    index: old_tile,
                    seed: old_tile as u64,
                    pos: Vec2 {
                        x: pos_u32.x as usize,
                        y: pos_u32.y as usize,
                    },
                });
            }
        }
    }

    fn get_percent_done(&self) -> f64 {
        let num_tiles = self.num_tiles as f64;
        let done = self.cur_index.load(Ordering::Relaxed) as f64;
        done / num_tiles
    }
}
