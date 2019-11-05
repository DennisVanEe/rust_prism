// TODO: get this sampler to work with the "new" sampler interface. Should be fairly straight forward

// use crate::math::numbers::Float;
// use crate::math::random::RandGen;
// use crate::math::vector::Vec2;
// use crate::memory::uninit_vec;
// use crate::sampler::Sampler;

// // The stratified sampler generates the sample values when initialized and stores
// // all of them in memory.
// // This will have to be somewhat dynamic when we get adaptive sampling working.

// pub struct Stratified {
//     // The number of pixel samples we will use per pixel
//     num_pixel_samples: usize,
//     curr_pixel_sample: usize,

//     num_pixel_samples_x: usize,
//     num_pixel_samples_y: usize,

//     num_dimensions: usize,

//     // The samples values are stored contiguously in memory but represents
//     // a 2D array of values.
//     // They are represented as such (first index which dimension, then which pixel sample we care about):
//     //
//     // dim0: pixel_sample0, pixel_sample1, pixel_sample2, ...
//     // dim1: pixel_sample0, pixel_sample1, pixel_sample2, ...
//     // ...
//     samples_1d: Vec<f64>,
//     samples_2d: Vec<Vec2<f64>>,
//     // The current dimension we are at:
//     curr_1d_dim: usize,
//     curr_2d_dim: usize,

//     // This is used when arbitrary arrays are requested (so NOT get_1d or get_2d):
//     // This is stored a fairly similarly as we did with the samples above:
//     //
//     // dim0: [pixel_sample0_arr], [pixel_sample1_arr], [pixel_sample2_arr], ...
//     // dim1: [pixel_sample0_arr], [pixel_sample1_arr], [pixel_sample2_arr], ...
//     // dim2: [pixel_sample0_arr], [pixel_sample1_arr], [pixel_sample2_arr], ...
//     samples_1d_arrays: Vec<f64>,
//     samples_2d_arrays: Vec<Vec2<f64>>,
//     array_1d_lens: Vec<usize>,
//     array_2d_lens: Vec<usize>,
//     // The current array we are looking at:
//     curr_1d_array: usize,
//     curr_2d_array: usize,
//     // The index of the current array we are looking at:
//     curr_1d_array_index: usize,
//     curr_2d_array_index: usize,

//     // to generate better samples:
//     rng: RandGen,
// }

// impl Stratified {
//     // The number of pixel samples in the x and y directions when using this sampler:
//     pub fn new(
//         num_pixel_samples_x: usize,
//         num_pixel_samples_y: usize,
//         num_dimensions: usize,
//         // If you are requesting memory, store them all in a vector
//         // and give it to us:
//         array_1d_lens: Vec<usize>,
//         array_2d_lens: Vec<usize>,
//         rng: RandGen,
//     ) -> Self {
//         // We just allocate all of the values here, we don't actually generate the samples until
//         // we start sampling a pixel:
//         let num_pixel_samples = num_pixel_samples_x * num_pixel_samples_y;

//         // Allocate buffers for the "regular" samples_1d and samples_2d values:
//         let (samples_1d, samples_2d) = {
//             let num_samples = num_pixel_samples * num_dimensions;
//             unsafe { (uninit_vec(num_samples), uninit_vec(num_samples)) }
//         };

//         let (samples_1d_arrays, samples_2d_arrays) = {
//             // So, for EACH individual pixel sample, we have these sizes:
//             let num_samples_1d = array_1d_lens.iter().sum() * num_pixel_samples;
//             let num_samples_2d = array_2d_lens.iter().sum() * num_pixel_samples;
//             unsafe { (uninit_vec(num_samples_1d), uninit_vec(num_samples_2d)) }
//         };

//         Stratified {
//             num_pixel_samples,
//             curr_pixel_sample: 0,

//             num_pixel_samples_x,
//             num_pixel_samples_y,

//             num_dimensions,

//             samples_1d,
//             samples_2d,
//             curr_1d_dim: 0,
//             curr_2d_dim: 0,

//             samples_1d_arrays,
//             samples_2d_arrays,
//             array_1d_lens,
//             array_2d_lens,

//             curr_1d_array: 0,
//             curr_2d_array: 0,

//             curr_1d_array_index: 0,
//             curr_2d_array_index: 0,

//             rng,
//         }
//     }

//     // Utility function for shuffling stuff around. Notice how it takes a &mut self.
//     // This is because we use the sampler's RNG member.
//     // block_size: divide samples into blocks of block_size and shuffle based on these blocks
//     fn shuffle<T>(&mut self, samples: &mut [T], block_size: usize) {
//         let num_blocks = samples.len() / block_size;
//         for curr_block_index in 0..num_blocks {
//             // Randomly pick another block:
//             let swap_block_index = self
//                 .rng
//                 .uniform_u32_limit((num_blocks - curr_block_index) as u32)
//                 as usize;

//             // Pick out the two blocks we want to swap:
//             let (samples0, samples1) = samples.split_at_mut(swap_block_index * block_size);
//             let block0 = &mut samples0[(curr_block_index * block_size)..];
//             let block1 = &mut samples1[..block_size];
//             // Now we swap them:
//             block0.swap_with_slice(block1);
//         }
//     }

//     fn gen_stratified_1d(&mut self, samples_data: &mut [f64]) {
//         let inv_num_samples = 1. / (samples_data.len() as f64);
//         for (i, sample) in &mut samples_data.iter().enumerate() {
//             // Offset from start of the stratified block:
//             let offset = self.rng.uniform_f64();
//             *sample = f64::ONE_MINUS_EPS.min(inv_num_samples * (i as f64 + offset));
//         }
//     }

//     fn gen_stratified_2d(
//         &mut self,
//         samples_data: &mut [Vec2<f64>],
//         num_samples_x: usize,
//         num_samples_y: usize,
//     ) {
//         debug_assert!(samples_data.len() == num_samples_x * num_samples_y);

//         let inv_num_samples_x = 1. / (num_samples_x as f64);
//         let inv_num_samples_y = 1. / (num_samples_y as f64);
//         for (i, sample) in &mut samples_data.iter().enumerate() {
//             // Get the x and y index given i:
//             let x_index = i % num_samples_x;
//             let y_index = i / num_samples_x;

//             let x_offset = self.rng.uniform_f64();
//             let y_offset = self.rng.uniform_f64();

//             sample.x = f64::ONE_MINUS_EPS.min(inv_num_samples_x * (x_index as f64 + x_offset));
//             sample.y = f64::ONE_MINUS_EPS.min(inv_num_samples_y * (y_index as f64 + y_offset));
//         }
//     }

//     // Generates latin hypercube samples for 2 dimensions:
//     fn gen_lhs_2d(&mut self, samples_data: &mut [Vec2<f64>]) {
//         // Generate a random value in each dimension
//         let inv_num_samples = 1. / (samples_data.len() as f64);
//         for (i, sample) in &mut samples_data.iter().enumerate() {
//             sample.x =
//                 f64::ONE_MINUS_EPS.min((i as f64 + self.rng.uniform_f64()) * inv_num_samples);
//             sample.y =
//                 f64::ONE_MINUS_EPS.min((i as f64 + self.rng.uniform_f64()) * inv_num_samples);
//         }

//         // Now we shuffle them in place of dimesnion (so shuffle the x's in the dimension of x and the y's in the dimension of y):
//         for (i, sample) in &mut samples_data.iter().enumerate() {
//             // Pick a value to swap with:
//             {
//                 let other_index =
//                     i + self.rng.uniform_u32_limit((samples_data.len() - i) as u32) as usize;
//                 let other_sample = &mut samples_data[other_index].x;
//                 let temp = sample.x;
//                 sample.x = *other_sample;
//                 *other_sample = temp;
//             }

//             {
//                 let other_index =
//                     i + self.rng.uniform_u32_limit((samples_data.len() - i) as u32) as usize;
//                 let other_sample = &mut samples_data[other_index].y;
//                 let temp = sample.y;
//                 sample.y = *other_sample;
//                 *other_sample = temp;
//             }
//         }
//     }
// }

// impl Sampler for Stratified {
//     // This sampler doesn't care which pixel we are sampling:
//     fn start_pixel(&mut self, _: Vec2<usize>) {
//         // Loop over and generate stratified samples for each of the pixel samples. This way,
//         // we make sure to hit each "region" of a pixel when sampling it:
//         for dim in 0..self.num_dimensions {
//             // Loop over each dimension:
//             let start_index = (dim * self.num_pixel_samples) as usize;
//             let end_index = start_index + (self.num_pixel_samples as usize);
//             let pixel_samples = &mut self.samples_1d[start_index..end_index];
//             self.gen_stratified_1d(pixel_samples);
//             // Now we want to shuffle this around:
//             self.shuffle(pixel_samples, 1);
//         }

//         // Do the same for the 2D samples:
//         for dim in 0..self.num_dimensions {
//             let start_index = (dim * self.num_pixel_samples) as usize;
//             let end_index = start_index + (self.num_pixel_samples as usize);
//             let pixel_samples = &mut self.samples_2d[start_index..end_index];
//             self.gen_stratified_2d(
//                 pixel_samples,
//                 self.num_pixel_samples_x,
//                 self.num_pixel_samples_y,
//             );
//             // Now we want to shuffle this around:
//             self.shuffle(pixel_samples, 1);
//         }

//         // We need to generate stratified samples for each of these values:
//         {
//             let mut curr_index = 0;
//             for &array_size in &self.array_1d_lens {
//                 for _ in 0..self.num_pixel_samples {
//                     let start_index = curr_index;
//                     let end_index = curr_index + array_size;
//                     let array = &mut self.samples_1d_arrays[start_index..end_index];
//                     self.gen_stratified_1d(array);
//                     curr_index = end_index;
//                 }
//             }

//             let mut curr_index = 0;
//             for &array_size in &self.array_2d_lens {
//                 for _ in 0..self.num_pixel_samples {
//                     let start_index = curr_index;
//                     let end_index = curr_index + array_size;
//                     let array = &mut self.samples_2d_arrays[start_index..end_index];
//                     self.gen_lhs_2d(array);
//                     curr_index = end_index;
//                 }
//             }
//         }
//     }

//     fn next_pixel_sample(&mut self) -> bool {
//         if self.curr_pixel_sample == self.num_pixel_samples - 1 {
//             return false;
//         }

//         self.curr_pixel_sample += 1;

//         // Zero information regarding our current position in the sampler:

//         self.curr_1d_dim = 0;
//         self.curr_2d_dim = 0;
//         self.curr_1d_array = 0;
//         self.curr_2d_array = 0;
//         self.curr_1d_array_index = 0;
//         self.curr_2d_array_index = 0;

//         true
//     }

//     fn get_num_pixel_samples(&self) -> usize {
//         self.num_pixel_samples
//     }

//     fn get_1d_array(&mut self) -> &[f64] {
//         // Check if we have more slices to deal with:
//         if self.curr_1d_array == self.samples_1d_arrays.len() {
//             return <&[f64]>::default();
//         }

//         // First figure out where the array we want is:
//         let array_len = self.array_1d_lens[self.curr_1d_array];
//         let array_start = self.curr_1d_array_index + array_len * self.curr_pixel_sample;
//         let array_end = array_start + array_len;

//         // Update the values as appropriate:
//         self.curr_1d_array += 1;
//         self.curr_1d_array_index += array_len * self.num_pixel_samples;

//         &self.samples_1d_arrays[array_start..array_end]
//     }

//     fn get_2d_array(&mut self) -> &[Vec2<f64>] {
//         // Check if we have more slices to deal with:
//         if self.curr_2d_array == self.samples_2d_arrays.len() {
//             return <&[Vec2<f64>]>::default();
//         }

//         // First figure out where the array we want is:
//         let array_len = self.array_2d_lens[self.curr_2d_array];
//         let array_start = self.curr_2d_array_index + array_len * self.curr_pixel_sample;
//         let array_end = array_start + array_len;

//         // Update the values as appropriate:
//         self.curr_2d_array += 1;
//         self.curr_2d_array_index += array_len * self.num_pixel_samples;

//         &self.samples_2d_arrays[array_start..array_end]
//     }

//     fn get_1d(&mut self) -> f64 {
//         // Check if more dimensions are used then we have access to:
//         if self.curr_1d_dim == self.num_dimensions {
//             return self.rng.uniform_f64();
//         }

//         let index = (self.curr_1d_dim * self.num_pixel_samples) + self.curr_pixel_sample;
//         let sample = self.samples_1d[index];
//         self.curr_1d_dim += 1;
//         sample
//     }

//     fn get_2d(&mut self) -> Vec2<f64> {
//         // Check if more dimensions are used then we have access to:
//         if self.curr_2d_dim == self.num_dimensions {
//             return Vec2 {
//                 x: self.rng.uniform_f64(),
//                 y: self.rng.uniform_f64(),
//             };
//         }

//         let index = (self.curr_2d_dim * self.num_pixel_samples) + self.curr_pixel_sample;
//         let sample = self.samples_2d[index];
//         self.curr_2d_dim += 1;
//         sample
//     }
// }
