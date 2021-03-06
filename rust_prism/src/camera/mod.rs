pub mod perspective;

use pmath::ray::{PrimaryRay, Ray, RayDiff};
use pmath::vector::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct CameraSample {
    /// Point on the film in raster space (pixel coordinate space)
    pub p_film: Vec2<f64>,
    /// Random uniform sample value used for generating a sample on the lense
    /// Range is [0, 1)
    pub p_lens: Vec2<f64>,
    /// Whatever time step is associated with this point.
    pub time: f64,
}

pub trait Camera: Send + Sync {
    /// Generates a single outgoing ray given a camera sample.
    fn gen_ray(&self, sample: CameraSample) -> Ray<f64>;

    /// Generates a primary ray, which is a ray with a dx and dy component for anti-aliasing
    ///
    /// Default implementation just uses the gen_ray function to generate dx and dy rays. These rays
    /// are generated by offseting the camera sample by one pixel in the x and y direction, respectively.
    fn gen_primary_ray(&self, sample: CameraSample) -> PrimaryRay<f64> {
        let ray = self.gen_ray(sample);

        // Generates a CameraSample that is shifted in the x direction:
        let xshift_sample = CameraSample {
            p_film: Vec2 {
                x: sample.p_film.x + 1.,
                y: sample.p_film.y,
            },
            p_lens: sample.p_lens,
            time: sample.time,
        };
        let rx = self.gen_ray(xshift_sample);

        // Generates a CameraSample that is shifted in the y direction:
        let yshift_sample = CameraSample {
            p_film: Vec2 {
                x: sample.p_film.x,
                y: sample.p_film.y + 1.,
            },
            p_lens: sample.p_lens,
            time: sample.time,
        };
        let ry = self.gen_ray(yshift_sample);

        PrimaryRay {
            ray,
            ray_diff: RayDiff {
                rx_org: rx.org,
                rx_dir: rx.dir,
                ry_org: ry.org,
                ry_dir: ry.dir,
            },
        }
    }
}
