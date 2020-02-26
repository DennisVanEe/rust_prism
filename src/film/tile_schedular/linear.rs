use crate::math::vector::Vec2;
use crate::math::util;

use super::super::TileIndex;
use super::TileSchedular;

use std::sync::atomic::{Ordering, AtomicUsize};

// A very basic linear tile schedular that just traverses the entire tile:
pub struct LinearTileSchedular {
    tile_len: usize,
    cur_index: AtomicUsize,
}

impl TileSchedular for LinearTileSchedular {
    // The number of times to traverse the tiles:
    type Param = ();

    fn new(tile_res: Vec2<usize>, _: ()) -> Self {
        let tile_len = tile_res.x * tile_res.y;
        LinearTileSchedular {
            tile_len,
            cur_index: AtomicUsize::new(0),
        }
    }

    fn reset(&mut self) {
        self.cur_index.store(0, Ordering::Relaxed);
    }

    fn init_index(&mut self) -> Option<TileIndex> {
        // Because it doesn't have to be thread safe:
        let index = self.cur_index.load(Ordering::Relaxed);
        if index < self.tile_len {
            // Let them out in morton order:
            self.cur_index.store(index + 1, Ordering::Relaxed);
            Some(TileIndex {
                index,
                pixel_pos: util::morton_to_2d(index as u64),
            })
        } else {
            None
        }
    }

    fn next_index(&self, _: TileIndex) -> Option<TileIndex> {
        // Get the current tile we have:
        let mut old_tile = self.cur_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.num_tiles {
                return None;
            } else {
                old_tile + 1
            };

            if let Err(i) = self.cur_index.compare_exchange_weak(old_tile, new_tile, Ordering::Relaxed, Ordering::Relaxed) {
                // Someone else changed the value, oh well, try again with a different i value:
                old_tile = i;
            } else {
                // We return the "old_tile". The new_tile is for the next time we run the code:
                return Some(TileIndex { 
                    index: old_tile,
                    pixel_pos: util::morton_to_2d(old_tile as u64),
                });
            }
        }
    }

    fn get_percent_done(&self) -> f64 {
        let tile_len = self.tile_len as f64;
        let done = self.cur_index.load(Ordering::Relaxed) as f64;
        done / tile_len
    }
}
