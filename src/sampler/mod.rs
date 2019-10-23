pub mod stratified;
pub mod zero_two;

use crate::camera::CameraSample;
use crate::math::numbers::Float;
use crate::math::random::RandGen;
use crate::math::vector::{Vec2, Vec3};

// Each thread, when working on a tile, gets access to their own
// sampler when rendering.

pub trait Sampler {
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
