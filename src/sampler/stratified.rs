use crate::sampler::Sampler;
use crate::math::vector::Vec2;
use crate::math::random::RandGen;
use crate::math::numbers::Float;

// The stratified sampler generates the sample values when initialized and stores
// all of them in memory.
// This will have to be somewhat dynamic when we get adaptive sampling working.

pub struct Stratified {
    // The number of pixel samples we will use per pixel
    num_pixel_samples: u64,
    num_pixel_samples_x: u64,
    num_pixel_samples_y: u64,
    curr_pixel_sample: u64,

    num_dimensions: u64,

    // The samples values are stored contiguously in memory but represents
    // a 2D array of values.
    // They are represented as such (first index which dimension, then which pixel sample we care about):
    //
    // dim0: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // dim1: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // ...
    samples_1d: Vec<f64>,
    samples_2d: Vec<Vec2<f64>>,

    rng: RandGen,
}

impl Stratified {
    // The number of pixel samples in the x and y directions when using this sampler:
    pub fn new(num_pixel_samples_x: u64, num_pixel_samples_y: u64, num_dimensions: u64, rng: RandGen) -> Self {
        // We just allocate all of the values here, we don't actually generate the samples until
        // we start sampling a pixel:
        let num_pixel_samples = num_pixel_samples_x * num_pixel_samples_y;
        let num_samples = (num_pixel_samples * num_dimensions) as usize;
        // We are allocating a large chunk of memory once (hence the unsafe code) so we
        // don't have to alocate this memory during rendering. I could allocate it 
        // on the first call to start pixel, but then I need more complex logic and I'm lazy.
        let mut samples_1d = Vec::with_capacity(num_samples);
        unsafe { samples_1d.set_len(num_samples); }
        let mut samples_2d = Vec::with_capacity(num_samples);
        unsafe { samples_2d.set_len(num_samples); }

        Stratified {
            num_pixel_samples,
            num_pixel_samples_x,
            num_pixel_samples_y,
            curr_pixel_sample: 0,
            num_dimensions,
            samples_1d,
            samples_2d,
            rng,
        }
    }

    // Utility function for shuffling stuff around. Notice how it takes a &mut self.
    // This is because we use the sampler's RNG member.
    // block_size: divide samples into blocks of block_size and shuffle based on these blocks
    fn shuffle<T>(&mut self, samples: &mut [T], block_size: usize) {
        let num_blocks = samples.len() / block_size;
        for curr_block_index in 0..num_blocks {
            // Randomly pick another block:
            let swap_block_index = self.rng.uniform_u32_limit((num_blocks - curr_block_index) as u32) as usize;

            // Pick out the two blocks we want to swap:
            let (samples0, samples1) = samples.split_at_mut(swap_block_index * block_size);
            let block0 = &mut samples0[(curr_block_index * block_size)..];
            let block1 = &mut samples1[..block_size];
            // Now we swap them:
            block0.swap_with_slice(block1);
        }
    }
}

impl Sampler for Stratified {
    // This sampler doesn't care which pixel we are sampling:
    fn start_pixel(&mut self, _: Vec2<usize>) {
        // Loop over and generate stratified samples for each of the pixel samples. This way,
        // we make sure to hit each "region" of a pixel when sampling it:
        let inv_num_samples = 1. / (self.num_pixel_samples as f64);
        for dim in 0..self.num_dimensions {
            // Loop over each dimension:
            let start_index = (dim * self.num_pixel_samples) as usize;
            let end_index = start_index + (self.num_pixel_samples as usize);
            let pixel_samples = &mut self.samples_1d[start_index..end_index];
            for (i, pixel_sample) in pixel_samples.iter_mut().enumerate() {
                // Offset from start of the stratified block:
                let offset = self.rng.uniform_f64();
                *pixel_sample = f64::ONE_MINUS_EPS.min(inv_num_samples * (i as f64 + offset));
            }
            // Now we want to shuffle this around:
            self.shuffle(pixel_samples, 1);
        }

        // Do the same for the 2D samples:
        let inv_num_samples_x = 1. / (self.num_pixel_samples_x as f64);
        let inv_num_samples_y = 1. / (self.num_pixel_samples_y as f64);
        for dim in 0..self.num_dimensions {
            let start_index = (dim * self.num_pixel_samples) as usize;
            let end_index = start_index + (self.num_pixel_samples as usize);
            let pixel_samples = &mut self.samples_2d[start_index..end_index];
            for (i, pixel_sample) in pixel_samples.iter_mut().enumerate() {
                // Get the x and y index given i:
                let x_index = i % self.num_pixel_samples_x as usize;
                let y_index = i / self.num_pixel_samples_x as usize;

                let x_offset = self.rng.uniform_f64();
                let y_offset = self.rng.uniform_f64();

                pixel_sample.x = f64::ONE_MINUS_EPS.min(inv_num_samples_x * (x_index as f64 + x_offset));
                pixel_sample.y = f64::ONE_MINUS_EPS.min(inv_num_samples_y * (y_index as f64 + y_offset));
            }
            // Now we want to shuffle this around:
            self.shuffle(pixel_samples, 1);
        }


    }


}