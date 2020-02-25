use crate::spectrum::{RGBSpectrum, XYZColor};
use crate::math::vector::{Vec2, Vec3};
use crate::memory;
use crate::math::util;

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

const TILE_DIM: usize = 8;
const TILE_LEN: usize = TILE_DIM * TILE_DIM;

struct AOVBuffer<'a, P: AOV> {
    buff: &'a [[P; TILE_LEN]],
}

impl<'a, P: AOV> AOVBuffer<'a, P> {
    pub fn get_tile(&self, tile_index: TileIndex) -> [P; TILE_LEN] {
        self.buff[tile_index.index]
    }

    // Make sure that tile isn't a reference from the buffer itself:
    pub fn set_tile(&self, tile_index: TileIndex, tile: &[P; TILE_LEN]) {
        let mut_buff: &'a mut [[P; TILE_LEN]] = unsafe {
            mem::transmute(self.buff)
        };
        mut_buff[tile_index.index] = tile.clone();
    }

    // Performs an addition over the aov values of the tile:
    pub fn update_tile(&self, tile_index: TileIndex, tile: &[P; TILE_LEN]) {
        let mut_buff: &'a mut [[P; TILE_LEN]] = unsafe {
            mem::transmute(self.buff)
        };
        let dst_tile = &mut mut_buff[tile_index.index];
        for (dst, src) in dst_tile.iter_mut().zip(tile.iter()) {
            dst.update(src);
        }
    }
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

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct Film<O: TileOrdering> {
    // List out all of the different types of AOV Buffers here:
    aov_buffers: EnumMap<AOVType, Option<Vec<u8>>>,

    ordering: O,              // The order in which we visit each tile
    tile_res: Vec2<usize>,    // The resolution in terms of tiles
    aov_res: Vec2<usize>,     // The resolution in terms of pixels
    num_tiles: usize,         // The total number of tiles here
    tile_index: AtomicUsize,  // A simple atomic counter that counts to the max value of data
}

impl<O: TileOrdering> Film<O> {
    pub fn new(tile_res: Vec2<usize>) -> Self {
        Film {
            aov_buffers: EnumMap::new(),
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

    // As per the request of the render, add AOV Films as we see fit:
    pub fn add_aov<P: AOV>(&mut self, init: P) {
        let buff = vec![[init; TILE_LEN]; self.num_tiles];
        let byte_buff: Vec<u8> = unsafe {
            memory::transmute_vec(buff)
        };
        self.aov_buffers[P::Type] = Some(byte_buff);
    }

    // Used to check if the AOV is present or not. If you also want to get
    // access to it afterwards, just call get_aovfilm().
    pub fn has_aov<P: AOV>(&self) -> bool {
        self.aov_buffers[P::Type].is_some()
    }

    // If an integrator wants to add stuff to a certain AOVFIlm, it must first
    // retrieve it through this function:
    pub fn get_aovfilm<P: AOV>(&self) -> Option<AOVBuffer<P>> {
        if let Some(byte_buff) = &self.aov_buffers[P::Type] {
            let buff_ptr = byte_buff.as_ptr() as *const [P; TILE_LEN];
            let buff = unsafe {
                slice::from_raw_parts(buff_ptr, self.num_tiles)
            };
            Some(AOVBuffer {
                buff
            })
        } else {
            None
        }
    }

    // Sets the entire aov buffer to the aov's init value. Returns true on success,
    // false if that aov buffer isn't present:
    pub fn set_init<P: AOV>(&mut self) -> bool {
        if let Some(byte_buff) = &mut self.aov_buffers[P::Type] {
            let buff_ptr = byte_buff.as_mut_ptr() as *mut [P; TILE_LEN];
            let buff = unsafe {
                slice::from_raw_parts_mut(buff_ptr, self.num_tiles)
            };
            // Iterate over the buffer and init all the pixels:
            for tile in buff.iter_mut() {
                for aov in tile.iter_mut() {
                    aov.set_init();
                }
            }
            true
        } else {
            false
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
