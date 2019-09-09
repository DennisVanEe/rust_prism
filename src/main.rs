mod camera;
mod film;
mod filter;
mod geometry;
mod math;
mod memory;
mod scene_loading;
mod spectrum;
mod transform;

use math::random::RNG;
use math::vector::Vec2;

fn main() {
    let gaus_filt = filter::GaussianFilter::new(Vec2 { x: 1., y: 1. }, 100.);
    let pixel_filter = film::PixelFilter::new(&gaus_filt);

    let mut rng = RNG::new_default();

    for _ in 0..1000 {
        let r1 = rng.uniform_f64();
        let r2 = rng.uniform_f64();

        let sample = pixel_filter.sample_pos(r1, r2);

        println!("{},{}", sample.x, sample.y);
    }
}
