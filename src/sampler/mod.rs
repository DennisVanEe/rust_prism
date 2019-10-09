pub mod stratified;

use crate::camera::CameraSample;
use crate::math::numbers::Float;
use crate::math::vector::{Vec2, Vec3};

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

pub fn sample_unit_disk(u: Vec2<f64>) -> Vec2<f64> {
    let r = u.x.sqrt();
    let theta = 2. * f64::PI * u.y;
    let (sin_theta, cos_theta) = theta.sin_cos();
    Vec2 {
        x: r * cos_theta,
        y: r * sin_theta,
    }
}

pub fn sample_concentric_disk(u: Vec2<f64>) -> Vec2<f64> {
    // First we map u to [-1, 1]x[-1, 1] square:
    let u_offset = u.scale(2.) - Vec2 { x: 1., y: 1. };
    if u_offset.x == 0. && u_offset.y == 0. {
        return Vec2::zero();
    }

    // The actual concentric mapping we want to do:
    let (theta, r) = if u_offset.x.abs() > u_offset.y.abs() {
        (f64::PI_OVER_4 * (u_offset.y / u_offset.x), u_offset.x)
    } else {
        (
            f64::PI_OVER_2 - f64::PI_OVER_4 * (u_offset.x / u_offset.y),
            u_offset.y,
        )
    };

    let (sin_theta, cos_theta) = theta.sin_cos();
    Vec2 {
        x: r * cos_theta,
        y: r * sin_theta,
    }
}

// This samples the hemisphere uniformly:
pub fn sample_uniform_hemisphere(u: Vec2<f64>) -> Vec3<f64> {
    let z = u.x;
    let r = (1. - z * z).max(0.).sqrt();
    let phi = 2. * f64::PI * u.y;
    let (sin_phi, cos_phi) = phi.sin_cos();
    Vec3 {
        x: r * cos_phi,
        y: r * sin_phi,
        z,
    }
}

// Regardless of where we are, the pdf is the same (as it's uniform)
pub fn pdf_uniform_hemisphere() -> f64 {
    f64::INV_2PI
}

// This just applies Malley's Method:
pub fn sample_cos_hemisphere(u: Vec2<f64>) -> Vec3<f64> {
    // We essentially need to map a point on the disk to the hemisphere:
    let d = sample_unit_disk(u);
    // x and y are trivial. z can be found using the Jacobian for a change of basis:
    let z = (1. - d.x * d.x - d.y * d.y).max(0.).sqrt();
    Vec3::from_vec2(d, z)
}

// We also want a corresponding pdf (with respect to omega (the solid angle)):
pub fn pdf_cos_hemisphere(cos_theta: f64) -> f64 {
    cos_theta * f64::INV_PI
}
