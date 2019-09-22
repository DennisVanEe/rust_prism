pub mod lambertian;
pub mod oren_nayar;
pub mod specular;
//pub mod microfacet;

use crate::math::vector::{Vec2, Vec3};
use crate::spectrum::RGBSpectrum;

use num_traits::clamp;

use bitflags::bitflags;

bitflags! {
    pub struct LobeType : u32 {
        const REFLECTION = 1 << 0;
        const TRANSMISSION = 1 << 1;
        const DIFFUSE = 1 << 2;
        const GLOSSY = 1 << 3;
        const SPECULAR = 1 << 4;
        const ALL = Self::REFLECTION.bits |
            Self::TRANSMISSION.bits | Self::DIFFUSE.bits |
            Self::GLOSSY.bits | Self::SPECULAR.bits;
    }
}

// This is a trait that represents BRDF (reflections) and BTDF (transmissions).
pub trait Lobe {
    // Returns whether or not the lobe has these types present.
    // This will be redundant as hell, but rust does not support fields
    // in traits.
    fn has_type(&self, fl: LobeType) -> bool;
    // Evaluates the lobe:
    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum;
    // sample_f is for sampling the f value and also works when we have a delta function
    // (for instance, with perfectly specular surfaces).
    fn sample_f(&self, wo: Vec3<f64>, sample: Vec2<f64>) -> (RGBSpectrum, Vec3<f64>, f64) {}
    // Returns the pdf in this case:
    fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> f64 {
        0.
    }
    // Used when calculating the hemispherical-directional reflectance:
    // Though, to calculate this value, we would need some samples (for some cases):
    fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>]) -> RGBSpectrum;
    // This performs the same calculation, but over the entire hemisphere:
    fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>]) -> RGBSpectrum;
}

// These functions assume one is currently in the shading space (that is, the normal is
// {0, 0, 1}).

fn cos_theta(w: Vec3<f64>) -> f64 {
    w.z
}

fn cos2_theta(w: Vec3<f64>) -> f64 {
    w.z * w.z
}

fn abs_cos_theta(w: Vec3<f64>) -> f64 {
    w.z.abs()
}

fn sin2_theta(w: Vec3<f64>) -> f64 {
    (1. - cos2_theta(w)).max(0.)
}

fn sin_theta(w: Vec3<f64>) -> f64 {
    sin2_theta(w).sqrt()
}

fn cos_phi(w: Vec3<f64>) -> f64 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0. {
        1.
    } else {
        clamp(w.x / sin_theta, -1., 1.)
    }
}

fn sin_phi(w: Vec3<f64>) -> f64 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0. {
        0.
    } else {
        clamp(w.y / sin_theta, -1., 1.)
    }
}

fn cos2_phi(w: Vec3<f64>) -> f64 {
    let cos_phi = cos_phi(w);
    cos_phi * cos_phi
}

fn sin2_phi(w: Vec3<f64>) -> f64 {
    let sin_phi = sin_phi(w);
    sin_phi * sin_phi
}

fn cos_dphi(w0: Vec3<f64>, w1: Vec3<f64>) -> f64 {
    let w0 = Vec2::from_vec3(w0);
    let w1 = Vec2::from_vec3(w1);
    let v = w0.dot(w1) / (w0.length2() * w1.length2()).sqrt();
    clamp(v, -1., 1.)
}

fn tan_theta(w: Vec3<f64>) -> f64 {
    sin_theta(w) / cos_theta(w)
}

fn tan2_theta(w: Vec3<f64>) -> f64 {
    sin2_theta(w) / cos2_theta(w)
}
