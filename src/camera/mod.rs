use crate::math::ray::{Ray, RayDiff};
use crate::math::vector::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct CameraSample {
    p_film: Vec2<f64>, // Where on the film this sample is present
    p_lens: Vec2<f64>, // Where on the lens this sample is present
    time: f64,         // The time when this sample is to be taken
}

pub trait Camera {
    fn generate_ray(&self, sample: CameraSample) -> Option<(Ray<f64>, f64)>;

    // If the weight is zero, then no ray is generated in this case:
    fn generate_raydiff(&self, sample: CameraSample) -> Option<(Ray<f64>, RayDiff<f64>, f64)> {
        let (ray, weight) = match self.generate_ray(sample) {
            Some(r) => r,
            _ => return None,
        };

        let xshift_sample = CameraSample {
            p_film: Vec2 {
                x: sample.p_film.x + 1.,
                y: sample.p_film.y,
            },
            p_lens: sample.p_lens,
            time: sample.time,
        };
        let (rx, _) = match self.generate_ray(xshift_sample) {
            Some(r) => r,
            _ => return None,
        };

        let yshift_sample = CameraSample {
            p_film: Vec2 {
                x: sample.p_film.x,
                y: sample.p_film.y + 1.,
            },
            p_lens: sample.p_lens,
            time: sample.time,
        };
        let (ry, _) = match self.generate_ray(yshift_sample) {
            Some(r) => r,
            _ => return None,
        };

        Some((ray, RayDiff { rx, ry }, weight))
    }
}
