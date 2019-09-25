use crate::math::vector::Vec2;
use crate::camera::CameraSample;

const ONE_MINUS_EPS: f64 = 0.99999999999999989;

pub trait Sampler {
    fn start_pixel(&mut self, pixel: Vec2<usize>);
    
    // Returns values for samples:

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
        CameraSample { p_film, p_lens, time }
    }
}