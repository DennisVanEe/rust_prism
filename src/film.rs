use crate::spectrum::{RGBSpectrum, XYZColor};
use crate::math::vector::{Vec2, Vec3};
use crate::memory;
use crate::math::util;

use std::cell::UnsafeCell;
use std::sync::atomic::{Ordering, AtomicUsize};
use std::slice;
use std::mem;

use enum_map::{Enum, EnumMap};

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
// AOV & AOVBuffer
//

//
// AOV (Arbitrary Output Variable)
//

// Defines a pixel to be used for different AOV rendering techniques:

trait AOV: Copy {
    // The final output type of the image. After we run a
    // "finalize" function over the buffer this is the final
    // result of the buffer. Usually this can be the same type
    // as the pixel:
    type FinalOutput: Copy;
    // The type of AOV, used to help indicate which buffer to access:
    const Type: AOVType;

    // Create an instance of the pixel in the initial state:
    fn init() -> Self;
    // Sets the aov to the initial state:
    fn set_init(&mut self);
    // Update the current pixel given another pixel:
    fn update(&mut self, p: &Self);
    // Outputs a "final" result for the current pixel:
    fn finalize(&self) -> Self::FinalOutput;
}

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
enum AOVType {
    Beauty,
    ShadNorm,
    //GeomNorm,
}

//
// BeautyAOV. The final rendered image output result goes here.
//

#[derive(Clone, Copy)]
pub struct BeautyAOV {
    pub value: XYZColor,
    pub count: u32,
}

impl AOV for BeautyAOV {
    type FinalOutput = RGBSpectrum;
    const Type: AOVType = AOVType::Beauty;

    fn init() -> Self {
        BeautyAOV {
            value: XYZColor::zero(),
            count: 0,
        }
    }

    fn set_init(&mut self) {
        self.value = XYZColor::zero();
        self.count = 0;
    }

    fn update(&mut self, p: &Self) {
        // Relatively simple update function:
        self.value = self.value + p.value;
        self.count = self.count + p.count;
    }

    fn finalize(&self) -> Self::FinalOutput {
        // First we normalize the XYZColor value:
        let weight = 1. / (self.count as f64);
        let final_xyz = self.value.scale(weight);
        // Convert it to RGBColor space:
        RGBSpectrum::from_xyz(final_xyz)
    }
}

//
// BeautyAOV. The final rendered image output result goes here.
//

#[derive(Clone, Copy)]
pub struct ShadNormAOV {
    pub norm: Vec3<f64>,
    pub count: u32,
}

impl AOV for ShadNormAOV {
    type FinalOutput = Vec3<f64>;
    const Type: AOVType = AOVType::ShadNorm;

    fn init() -> Self {
        ShadNormAOV {
            norm: Vec3::zero(),
            count: 0,
        }
    }

    fn set_init(&mut self) {
        self.norm = Vec3::zero();
        self.count = 0;
    }

    fn update(&mut self, p: &Self) {
        // Relatively simple update function:
        self.norm = self.norm + p.norm;
        self.count = self.count + p.count;
    }

    fn finalize(&self) -> Self::FinalOutput {
        // First we normalize the XYZColor value:
        let weight = 1. / (self.count as f64);
        self.norm.scale(weight)
    }
}

// Just a collection of the different AOV buffers available:
pub struct Film {
    pub beauty: Option<AOVBuffer<BeautyAOV>>,
    pub shad_norm: Option<AOVBuffer<ShadNormAOV>>,
}

impl Film {
    // Defaults to an empty film:
    pub fn new() -> Self {
        Film {
            beauty: None,
            shad_norm: None,
        }
    }
}

const TILE_DIM: usize = 8;
const TILE_LEN: usize = TILE_DIM * TILE_DIM;

pub struct AOVBuffer<P: AOV> {
    buff: UnsafeCell<Vec<[P; TILE_LEN]>>,
}

impl<P: AOV> AOVBuffer<P> {
    // Initializes a new buffer with the given tile resolution:
    pub fn new(tile_res: Vec2<usize>, init: P) -> Self {
        AOVBuffer {
            buff: UnsafeCell::new(vec![[init; TILE_LEN]; tile_res.x * tile_res.y]),
        }
    }

    // This is set to be unsafe because this isn't thread safe:
    pub fn set_entire(&mut self, init: P) {
        let buff = unsafe {
            &mut *self.buff.get()
        };
        for tile in buff.iter_mut() {
            for p in tile.iter_mut() {
                p.set_init();
            }
        }
    }

    pub fn get_tile(&self, tile_index: &TileIndex) -> [P; TILE_LEN] {
        let buff = unsafe {
            &*self.buff.get()
        };
        buff[tile_index.index]
    }

    // Make sure that tile isn't a reference from the buffer itself:
    pub fn set_tile(&self, tile_index: &TileIndex, tile: &[P; TILE_LEN]) {
        let buff = unsafe {
            &mut *self.buff.get()
        };
        buff[tile_index.index] = tile.clone();
    }

    // Performs an addition over the aov values of the tile:
    pub fn update_tile(&self, tile_index: &TileIndex, tile: &[P; TILE_LEN]) {
        let buff = unsafe {
            &mut *self.buff.get()
        };
        let dst_tile = &mut buff[tile_index.index];
        for (dst, src) in dst_tile.iter_mut().zip(tile.iter()) {
            dst.update(src);
        }
    }
}

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct TileSchedular<'a, O: TileOrdering> {
    // The TileSchedular needs access to the Film incase certain
    // schedules depend on the value of the aov (like for adaptive
    // sampling or something).
    pub film: &'a Film,

    // Stuff for determining tile order and whatnot:

    ordering: O,              // The order in which we visit each tile
    tile_res: Vec2<usize>,    // The resolution in terms of tiles
    aov_res: Vec2<usize>,     // The resolution in terms of pixels
    num_tiles: usize,         // The total number of tiles here
    tile_index: AtomicUsize,  // A simple atomic counter that counts to the max value of data
}

impl<'a, O: TileOrdering> TileSchedular<'a, O> {
    pub fn new(tile_res: Vec2<usize>, film: &'a Film) -> Self {
        TileSchedular {
            film,
            ordering: O::new(tile_res),
            tile_res,
            aov_res: tile_res.scale(TILE_DIM),
            num_tiles: tile_res.x * tile_res.y,
            tile_index: AtomicUsize::new(0),
        }
    }

    // Returns an initial tile index. This is the first tile that a thread will be rendering.
    // It takes a mutable self to indicate that this should NOT be called by individual threads,
    // it should only be called initially.
    pub fn get_init_tile_index(&mut self) -> TileIndex {
        // This operation doesn't necessarily have to be thread safe:
        let index = self.tile_index.fetch_add(1, Ordering::Relaxed);
        let tile_pos = self.ordering.get_pos(index);
        let aov_pos = tile_pos.scale(TILE_DIM);
        TileIndex {
            index,
            aov_pos
        }
    }

    // Consume the TileIndex so that the user can't use the same tile index again. Also,
    // when using a more complex tile ordering system, this means we know which tile we
    // sent back.
    pub fn next_tile_index(&self, _: TileIndex) -> Option<TileIndex> {
        // Get the current tile we have:
        let mut old_tile = self.tile_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.num_tiles {
                // When I'm working on adding adaptive sampling, I can change what the tile index should
                // be once I've gone through all possible options here:
                // 0
                return None;
            } else {
                old_tile + 1
            };

            if let Err(i) = self.tile_index.compare_exchange_weak(old_tile, new_tile, Ordering::Relaxed, Ordering::Relaxed) {
                // Someone else changed the value, oh well, try again with a different i value:
                old_tile = i;
            } else {
                // We return the "old_tile". The new_tile is for the next time we run the code:
                return Some(TileIndex { 
                    index: old_tile,
                    aov_pos: self.ordering.get_pos(old_tile),
                });
            }
        }
    }

    pub fn get_num_tiles(&self) -> usize {
        self.num_tiles
    }

    pub fn get_aov_res(&self) -> Vec2<usize> {
        self.aov_res
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.tile_res
    }
}

// A special type that can only be created in the film crate.
// No copy or clone is implemented for this. Once TileIndex is used
pub struct TileIndex {
    index: usize,         // Actual index of said tile
    aov_pos: Vec2<usize>, // Top left aov position
}

impl TileIndex {
    // Returns the top left corner pixel position of the given tile.
    // If none, then it's an invalid tile index and shouldn't be used.
    pub fn aov_pos(&self) -> Vec2<usize> {
        self.aov_pos
    }
}

