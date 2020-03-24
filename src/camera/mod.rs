pub mod perspective;

use crate::math::ray::{Ray, RayDiff};
use crate::math::vector::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct CameraSample {
    p_film: Vec2<f64>, // Where on the film this sample is present
    p_lens: Vec2<f64>, // Where on the lens this sample is present
    time: f64,
}

pub trait Camera {
    /// Generates a single outgoing ray
    /// 
    /// # Arguments
    /// * `sample` - A sample generated by a sampler used to generate the ray.
    fn generate_ray(&self, sample: CameraSample) -> Ray<f64>;

    /// Generates a single outgoing ray and two rays slightly offset in the 
    /// x and y direction called Ray Differentiables.
    /// 
    /// # Arguments
    /// * `sample` - A sample generated by a sampler used to generate the ray.
    fn generate_raydiff(&self, sample: CameraSample) -> (Ray<f64>, RayDiff<f64>) {
        let ray = self.generate_ray(sample);

        // Generates a CameraSample that is shifted in the x direction:
        let xshift_sample = CameraSample {
            p_film: Vec2 {
                x: sample.p_film.x + 1.,
                y: sample.p_film.y,
            },
            p_lens: sample.p_lens,
            time: sample.time,
        };
        let rx = self.generate_ray(xshift_sample);

        // Generates a CameraSample that is shifted in the y direction:
        let yshift_sample = CameraSample {
            p_film: Vec2 {
                x: sample.p_film.x,
                y: sample.p_film.y + 1.,
            },
            p_lens: sample.p_lens,
            time: sample.time,
        };
        let ry = self.generate_ray(yshift_sample);

        (ray, RayDiff { rx, ry })
    }
}
