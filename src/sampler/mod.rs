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
