pub mod stratified;
pub mod zero_two;

use crate::camera::CameraSample;
use crate::math::random::RandGen;
use crate::math::vector::Vec2;

// Each thread, when working on a tile, gets access to their own
// sampler when rendering.

// Also defines any extra parameters a sampler might want:
pub trait Sampler {
    // Parameter type, if any. Once default parameter
    // types aren't an unstable feature it'll be the empty tuple:
    type ParamType;

    fn new(
        // If the sampler requires extra parameters, pass them here:
        param: Self::ParamType,
        // The number of pixel samples:
        num_pixel_samples: usize,
        // The number of dimensions:
        num_dim: usize,
        // Not really a seed, but is used to define the random number generator:
        seed: u64,
        // If any arrays are to be requested for 1d, request them here:
        arr_sizes_1d: &[usize],
        // If any arrays are to be requested for 2d, request them here:
        arr_sizes_2d: &[usize],
    ) -> Self;

    // Use the sampler to start working on a new pixel:
    fn start_pixel(&mut self, pixel: Vec2<usize>);

    // This gets called constantly until it returns false.
    // When it does it means we have sampled the pixel sample number of times.
    fn next_pixel_sample(&mut self) -> bool;

    fn get_num_pixel_samples(&self) -> usize;

    // Returns values for samples:

    fn get_1d_array(&mut self) -> &[f64];
    fn get_2d_array(&mut self) -> &[Vec2<f64>];

    fn get_1d(&mut self) -> f64;
    fn get_2d(&mut self) -> Vec2<f64>;

    fn get_camera_sample(&mut self) -> CameraSample {
        // Because of the way we do filtering, we don't
        // care about the position relative to the entire film.
        // Instead, we care about the position relative to the
        // specific pixel.
        let p_film = self.get_2d();
        let time = self.get_1d();
        let p_lens = self.get_2d();
        CameraSample {
            p_film,
            p_lens,
            time,
        }
    }

    // Rounds a provided count to an appropriate count for the sampler
    // to work with (such as a power of 2, given that many samplers
    // prefer working with such values):
    fn round_count(cnt: usize) -> usize {
        cnt
    }
}

// This is used by numerous samplers, so we have it defined here:

fn shuffle<T>(samples: &mut [T], block_size: usize, rng: &mut RandGen) {
    let num_blocks = samples.len() / block_size;
    for curr_block_index in 0..num_blocks {
        // Randomly pick another block:
        let swap_block_index =
            rng.uniform_u32_limit((num_blocks - curr_block_index) as u32) as usize;

        // Pick out the two blocks we want to swap:
        let (samples0, samples1) = samples.split_at_mut(swap_block_index * block_size);
        let block0 = &mut samples0[(curr_block_index * block_size)..];
        let block1 = &mut samples1[..block_size];
        // Now we swap them:
        block0.swap_with_slice(block1);
    }
}
