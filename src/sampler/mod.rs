pub mod stratified;
pub mod zero_two;

use crate::camera::CameraSample;
use crate::filter::PixelFilter;
use crate::math::random::RandGen;
use crate::math::vector::Vec2;

// Each thread, when working on a tile, gets access to their own
// sampler when rendering.

// Also defines any extra parameters a sampler might want:
// Must also be clonable:
pub trait Sampler: Clone {
    // Some samplers may be better at generating samples if it knows the arrays it has to generate:
    fn prepare_arrays(&mut self, arr_sizes_1d: &[usize], arr_sizes_2d: &[usize]);

    /// Tells the sampler that we are starting on a new specific pixel.
    ///
    /// # Arguments
    /// * `pixel` - The pixel's top left position.
    fn start_pixel(&mut self, pixel: Vec2<usize>);

    /// Returns the current pixel's top left position.
    fn get_curr_pixel_pos(&self) -> Vec2<usize>;

    // Specify that we are starting a different tile. Need to
    // redefine a seed as this removes potential artifacting.
    // Note, that BEFORE EVERY PIXEL start_pixel is called. This means
    // that this function shouldn't perform start_pixel operations:
    fn start_tile(&mut self, seed: u64);

    // This gets called constantly until it returns false.
    // When it does it means we have sampled the pixel sample number of times.
    fn next_pixel_sample(&mut self) -> bool;

    fn get_num_pixel_samples(&self) -> usize;

    // Returns values for samples:

    fn get_1d_array(&mut self) -> &[f64];
    fn get_2d_array(&mut self) -> &[Vec2<f64>];

    fn get_1d(&mut self) -> f64;
    fn get_2d(&mut self) -> Vec2<f64>;

    /// Uses the sampler to generate a `CameraSample`. This would be passed into
    /// a camera to generate a `Ray` and `RayDiff`.
    ///
    /// # Arguments
    /// * `filter` - The pixel filter used to determine a point on the film.
    fn gen_camera_sample(&mut self, filter: &PixelFilter) -> CameraSample {
        // Sample a value for the specific pixel and offset it by the pixel's
        // actual position.
        let p_film = filter.sample_pos(self.get_2d()) + self.get_curr_pixel_pos();
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
