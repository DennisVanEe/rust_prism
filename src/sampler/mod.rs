pub mod stratified;

use crate::camera::CameraSample;
use crate::math::numbers::Float;
use crate::math::random::RandGen;
use crate::math::vector::Vec2;

const ONE_MINUS_EPS: f64 = 0.99999999999999989;

// Each thread, when working on a tile, gets access to their own
// sampler when rendering.

pub trait Sampler {
    // Use the sampler to start working on a new pixel:
    fn start_pixel(&mut self, pixel: Vec2<usize>);

    // This gets called constantly until it returns false.
    // When it does it means we have sampled the pixel sample number of times.
    fn start_next_sample(&mut self) -> bool;

    fn get_num_pixel_samples(&self) -> u64;

    // Returns values for samples:

    fn request_1d_array(&mut self, size: usize);
    fn request_2d_array(&mut self, size: usize);

    fn get_1d_array(&mut self, size: usize) -> &[f64];
    fn get_2d_array(&mut self, size: usize) -> &[Vec2<f64>];

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

// These functions may be useful for other samplers:

fn gen_stratified_samples_1d(samples_data: &mut [f64], num_samples: usize, rng: &mut RandGen) {
    let inv_num_samples = 1. / (num_samples as f64);
    for (i, sample) in samples_data.iter_mut().enumerate() {
        // Offset from start of the stratified block:
        let offset = rng.uniform_f64();
        *sample = f64::ONE_MINUS_EPS.min(inv_num_samples * (i as f64 + offset));
    }
}

fn gen_stratified_samples_2d(
    samples_data: &mut [Vec2<f64>],
    num_samples_x: usize,
    num_samples_y: usize,
    rng: &mut RandGen,
) {
    let inv_num_samples_x = 1. / (num_samples_x as f64);
    let inv_num_samples_y = 1. / (num_samples_y as f64);
    for (i, sample) in samples_data.iter_mut().enumerate() {
        // Get the x and y index given i:
        let x_index = i % num_samples_x;
        let y_index = i / num_samples_x;

        let x_offset = rng.uniform_f64();
        let y_offset = rng.uniform_f64();

        sample.x =
            f64::ONE_MINUS_EPS.min(inv_num_samples_x * (x_index as f64 + x_offset));
        sample.y =
            f64::ONE_MINUS_EPS.min(inv_num_samples_y * (y_index as f64 + y_offset));
    }
}