use crate::math::numbers::Float;
use crate::math::random::RandGen;
use crate::math::vector::Vec2;
use crate::sampler::{gen_stratified_samples_1d, gen_stratified_samples_2d, Sampler};

// The stratified sampler generates the sample values when initialized and stores
// all of them in memory.
// This will have to be somewhat dynamic when we get adaptive sampling working.

pub struct Stratified {
    // The number of pixel samples we will use per pixel
    num_pixel_samples: usize,
    curr_pixel_sample: usize,

    num_pixel_samples_x: usize,
    num_pixel_samples_y: usize,

    num_dimensions: usize,

    // The samples values are stored contiguously in memory but represents
    // a 2D array of values.
    // They are represented as such (first index which dimension, then which pixel sample we care about):
    //
    // dim0: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // dim1: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // ...
    samples_1d: Vec<f64>,
    samples_2d: Vec<Vec2<f64>>,
    // The current start index we are at (not the index of the dimension):
    curr_1d_dim: usize,
    curr_2d_dim: usize,

    // When requesting arbitrary number of samples, we store them here:
    array_1d_lens: Vec<usize>,
    array_2d_lens: Vec<usize>,

    samples_1d_array: Vec<f64>,
    samples_2d_array: Vec<Vec2<f64>>,

    // to generate better samples:
    rng: RandGen,
}

impl Stratified {
    // The number of pixel samples in the x and y directions when using this sampler:
    pub fn new(
        num_pixel_samples_x: usize,
        num_pixel_samples_y: usize,
        num_dimensions: usize,
        // If you are requesting memory, store them all in a vector
        // and give it to us:
        array_1d_lens: Vec<usize>,
        array_2d_lens: Vec<usize>,
        rng: RandGen,
    ) -> Self {
        // We just allocate all of the values here, we don't actually generate the samples until
        // we start sampling a pixel:
        let num_pixel_samples = num_pixel_samples_x * num_pixel_samples_y;
        let num_samples = num_pixel_samples * num_dimensions;
        // We are allocating a large chunk of memory once (hence the unsafe code) so we
        // don't have to alocate this memory during rendering. I could allocate it
        // on the first call to start pixel, but then I need more complex logic and I'm lazy.
        let mut samples_1d = Vec::with_capacity(num_samples);
        unsafe {
            samples_1d.set_len(num_samples);
        }
        let mut samples_2d = Vec::with_capacity(num_samples);
        unsafe {
            samples_2d.set_len(num_samples);
        }

        let num_array_samples = array_1d_lens.iter().sum();
        let mut samples_1d_array = Vec::with_capacity(num_array_samples);
        unsafe {
            samples_1d_array.set_len(num_array_samples);
        }
        let num_array_samples = array_2d_lens.iter().sum();
        let mut samples_2d_array = Vec::with_capacity(num_array_samples);
        unsafe {
            samples_2d_array.set_len(num_array_samples);
        }

        Stratified {
            num_pixel_samples,
            curr_pixel_sample: 0,

            num_pixel_samples_x,
            num_pixel_samples_y,

            num_dimensions,

            samples_1d,
            samples_2d,

            curr_1d_dim: 0,
            curr_2d_dim: 0,

            array_1d_lens,
            array_2d_lens,

            samples_1d_array,
            samples_2d_array,

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
            let swap_block_index = self
                .rng
                .uniform_u32_limit((num_blocks - curr_block_index) as u32)
                as usize;

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
        for dim in 0..self.num_dimensions {
            // Loop over each dimension:
            let start_index = (dim * self.num_pixel_samples) as usize;
            let end_index = start_index + (self.num_pixel_samples as usize);
            let pixel_samples = &mut self.samples_1d[start_index..end_index];
            gen_stratified_samples_1d(pixel_samples, self.num_pixel_samples, &mut self.rng);
            // Now we want to shuffle this around:
            self.shuffle(pixel_samples, 1);
        }

        // Do the same for the 2D samples:
        for dim in 0..self.num_dimensions {
            let start_index = (dim * self.num_pixel_samples) as usize;
            let end_index = start_index + (self.num_pixel_samples as usize);
            let pixel_samples = &mut self.samples_2d[start_index..end_index];
            gen_stratified_samples_2d(pixel_samples, self.num_pixel_samples_x, self.num_pixel_samples_y, &mut self.rng);
            // Now we want to shuffle this around:
            self.shuffle(pixel_samples, 1);
        }
    }

    fn start_next_sample(&mut self) -> bool {
        if self.curr_pixel_sample == self.num_pixel_samples - 1 {
            return false;
        }

        self.curr_pixel_sample += 1;
        // Zero the current dimension we are at:
        self.curr_1d_dim = 0;
        self.curr_2d_dim = 0;

        true
    }

    fn get_1d(&mut self) -> f64 {
        // Check if more dimensions are used then we have access to:
        if self.curr_1d_dim == self.num_dimensions {
            return self.rng.uniform_f64();
        }

        let index = (self.curr_1d_dim * self.num_pixel_samples) + self.curr_pixel_sample;
        let sample = unsafe { *self.samples_1d.get_unchecked(index) };
        self.curr_1d_dim += 1;
        sample
    }

    fn get_2d(&mut self) -> Vec2<f64> {
        // Check if more dimensions are used then we have access to:
        if self.curr_2d_dim == self.num_dimensions {
            return Vec2 {
                x: self.rng.uniform_f64(),
                y: self.rng.uniform_f64(),
            };
        }

        let index = (self.curr_2d_dim * self.num_pixel_samples) + self.curr_pixel_sample;
        let sample = unsafe { *self.samples_2d.get_unchecked(index) };
        self.curr_2d_dim += 1;
        sample
    }
}
