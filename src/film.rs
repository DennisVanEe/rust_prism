use crate::spectrum::{RGBSpectrum, XYZColor};
use crate::math::vector::{Vec2, Vec3};
use crate::memory;
use crate::math::util;

use std::sync::atomic::{Ordering, AtomicUsize};
use std::iter::IntoIterator;
use std::slice::{self, IterMut, Iter};
use std::ptr;
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

struct AOVBuffer {
    // It's a wrapper for different types of AOVs so that we can
    // store a collection of AOVBuffers without worrying about them
    // being heterogeneous.
    buffer: Vec<u8>,
    tile_dim: Vec2<usize>,
    tile_len: usize,
    aov_type: AOVType,
}

impl AOVBuffer {
    pub fn new<P: AOV>(tile_dim: Vec2<usize>, init: P) -> Self {
        let len = tile_dim.x * tile_dim.y * mem::size_of::<P>();
        let aov_buffer = vec![[init; TILE_LEN]; len];
        AOVBuffer {
            buffer: unsafe { memory::transmute_vec(aov_buffer) },
            tile_dim,
            tile_len: tile_dim.x * tile_dim.y,
            aov_type: P::Type,
        }
    }

    pub unsafe fn set_init<P: AOV>(&mut self) {
        debug_assert_eq!(self.aov_type, P::Type, "The type of the AOVBuffer and AOV::Type must match.");
        let buff_ptr: *mut P = mem::transmute(self.buffer.as_mut_ptr());
        let aov_buff = slice::from_raw_parts_mut(buff_ptr, self.buffer.len());
        for aov in aov_buff.iter_mut() {
            aov.set_init();
        }
    }

    pub unsafe fn get_tile<P: AOV>(&self, tile_index: usize) -> [P; TILE_LEN] {
        debug_assert_eq!(self.aov_type, P::Type, "The type of the AOVBuffer and AOV::Type must match.");
        let tile_size = mem::size_of::<[P; TILE_LEN]>();
        let byte_start = tile_index * tile_size;
        let byte_end = byte_start + tile_size;
        let tile_bytes = &self.buffer[byte_start..byte_end];
        ptr::read(tile_bytes.as_ptr() as *const _) 
    }

    // Make sure that tile isn't a reference from the buffer itself:
    pub unsafe fn set_tile<P: AOV>(&mut self, tile_index: usize, tile: &[P; TILE_LEN]) {
        debug_assert_eq!(self.aov_type, P::Type, "The type of the AOVBuffer and AOV::Type must match.");
        let tile_size = mem::size_of::<[P; TILE_LEN]>();
        let byte_start = tile_index * tile_size;
        let byte_end = byte_start + tile_size;
        let tile_bytes = &mut self.buffer[byte_start..byte_end];
        ptr::copy_nonoverlapping(tile.as_ptr() as *const P, tile_bytes.as_mut_ptr() as *mut P, tile_size);
    }

    // Performs an addition over the aov values of the tile:
    pub unsafe fn update_tile<P: AOV>(&mut self, tile_index: usize, tile: &[P; TILE_LEN]) {
        debug_assert_eq!(self.aov_type, P::Type, "The type of the AOVBuffer and AOV::Type must match.");
        let tile_size = mem::size_of::<[P; TILE_LEN]>();
        let byte_start = tile_index * tile_size;
        let byte_end = byte_start + tile_size;
        let tile_bytes = &mut self.buffer[byte_start..byte_end];
        // Construct a slice of type P:
        let dst_ptr: *mut P = mem::transmute(tile_bytes.as_mut_ptr());
        let dst_tile = slice::from_raw_parts_mut(dst_ptr, self.tile_len);
        for (dst_aov, src_aov) in dst_tile.iter_mut().zip(tile.iter()) {
            dst_aov.update(src_aov);
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

// We only make public what we want to expose to the user
// of this specific tile:
#[derive(Clone, Copy)]
pub struct TileIndexData {
    tile_index: usize, // This is the normal, scanline index of the tile
    pub tile_pos: Vec2<usize>, // The (x, y) coordinate of the tile if it were on film
    pub aov_pos: Vec2<usize>, // The (x, y) coordinate of the top left most aov value
}

// A special type that can only be created in the film crate.
// No copy or clone is implemented for this. Once TileIndex is used
pub struct TileIndex {
    data: Option<TileIndexData>,
}

impl TileIndex {
    // None means it's either an initial tile or we are done
    // rendering and there are no more tiles:
    pub fn get_data(&self) -> Option<TileIndexData> {
        self.data
    }
}

// Now, for that reason, data is not stored as a normal pixel buffer
// would be (it's not just a 2D array in a 1D array form):
pub struct Film<O: TileOrdering> {
    // List out all of the different types of AOV Buffers here:
    aov_buffers: EnumMap<AOVType, Option<AOVBuffer>>,

    ordering: O,                  // The order in which we visit each tile
    tile_res: Vec2<usize>,        // The resolution in terms of tiles
    aov_res: Vec2<usize>,       // The resolution in terms of pixels
    num_tiles: usize,
    curr_tile_index: AtomicUsize, // A simple atomic counter that counts to the max value of data
}

impl<O: TileOrdering> Film<O> {
    pub fn new(tile_res: Vec2<usize>) -> Self {
        Film {
            aov_buffers: EnumMap::new(),
            ordering: O::new(tile_res),
            tile_res,
            aov_res: tile_res.scale(TILE_DIM),
            num_tiles: tile_res.x * tile_res.y,
            curr_tile_index: AtomicUsize::new(0),
        }
    }

    // Allows someone to dynamically add them:
    pub fn add_aovbuff<P: AOV>(&mut self, init: P) {
        self.aov_buffers[P::Type] = Some(AOVBuffer::new(self.tile_res, init));
    }

    // Sets the entire aov buffer to the aov's init value. Returns true on success,
    // false if that aov buffer isn't present:
    pub fn set_init<P: AOV>(&mut self) -> bool {
        if let Some(buffer) = self.aov_buffers[P::Type] {
            unsafe { buffer.set_init::<P>(); }
            true
        } else {
            false
        }
    }

    // Returns the tile data present at the given tile_index. If the aov_buffer
    // for this operation doesn't exist, return None.
    pub fn get_tile<P: AOV>(&self, index: &TileIndex) -> Option<[P; TILE_LEN]> {
        if let Some(index_data) = index.data {
            if let Some(buffer) = self.aov_buffers[P::Type] {
                Some(buffer.get_tile::<P>(index_data.tile_index))
            } else {
                None
            }
        } else {
            // Instead of returning None we panic as this should NEVER
            // happen:
            panic!();
        }
    }

    // TODO: Instead of constantly checking if the aov buffer exists when performing an operation
    // on it, return some sort of handle to it that the thread can use. This way the check is only
    // performed once. May need to encapsulate this in something so the thread doesn't do anything
    // too stupid.

    // Given a tile, calls the update operation on it. Returns true if the operation was
    // complete. Returns false if the aov buffer doesn't exist:
    pub fn update_tile<P: AOV>(&self, index: &TileIndex, tile: &[P; TILE_LEN]) -> bool {
        if let Some(index_data) = index.data {
            if let Some(buffer) = self.aov_buffers[P::Type] {
                buffer.update_tile::<P>(index_data.tile_index, tile);
                true
            } else {
                false
            }
        } else {
            panic!();
        }
    }

    pub fn set_tile<P: AOV>(&self, index: &TileIndex, tile: &[P; TILE_LEN]) -> bool {
        if let Some(index_data) = index.data {
            if let Some(buffer) = self.aov_buffers[P::Type] {
                buffer.set_tile::<P>(index_data.tile_index, tile);
                true
            } else {
                false
            }
        } else {
            panic!();
        }
    }

    // Generates an initial tile index for rendering:
    pub fn init_tile_index() -> TileIndex {
        TileIndex {
            data: None,
        }
    }

    // Consume the TileIndex so that the user can't use the same tile index again. Also,
    // when using a more complex tile ordering system, this means we know which tile we
    // sent back.
    pub fn next_tile_index(&self, curr_tile_index: TileIndex) -> TileIndex {
        // Get the current tile we have:
        let mut old_tile = self.curr_tile_index.load(Ordering::Relaxed);
        loop {
            // Check if this tile is already at the max. If it is, then we are done.
            let new_tile = if old_tile >= self.num_tiles {
                // When I'm working on adding adaptive sampling, I can change what the tile index should
                //  be once I've gone through all possible options here:
                // 0
                return TileIndex { index: None };
            } else {
                old_tile + 1
            };

            if let Err(x) = self.curr_tile_index.compare_exchange_weak(old_tile, new_tile, Ordering::Relaxed, Ordering::Relaxed) {
                // Someone else changed the value, oh well, try again with a different x value:
                old_tile = x;
            } else {
                // We return the "old_tile". The new_tile is for the next time we run the code:
                return TileIndex { index: Some(old_tile) };
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
