pub mod lambertian;
pub mod oren_nayar;
pub mod specular;
pub mod microfacet;

use crate::math::numbers::Float;
use crate::math::vector::{Vec2, Vec3};
use crate::sampler::{
    pdf_cos_hemisphere, pdf_uniform_hemisphere, sample_cos_hemisphere, sample_uniform_hemisphere,
};
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
    fn matches_type(&self, lobe_type: LobeType) -> bool;
    // Returns the lobe type:
    fn get_type(&self) -> LobeType;
    // Evaluates the lobe:
    fn eval(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum;
    // sample_f is for sampling the f value and also works when we have a delta function
    // (for instance, with perfectly specular surfaces).
    fn sample(&self, wo: Vec3<f64>, u: Vec2<f64>) -> (RGBSpectrum, Vec3<f64>, f64) {
        // We have to flip wi as we are dealign with shading coordinate system.
        // If we had to flip it, that means the normal is on the other side of wo.
        let wi = if wo.z < 0. {
            let p = sample_cos_hemisphere(u);
            Vec3 {
                x: p.x,
                y: p.y,
                z: -p.z,
            }
        } else {
            sample_cos_hemisphere(u)
        };
        let pdf = self.pdf(wo, wi);
        let eval = self.eval(wo, wi);
        (eval, wi, pdf)
    }
    // Returns the pdf in this case:
    fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> f64 {
        if is_in_same_hemisphere(wo, wi) {
            pdf_cos_hemisphere(abs_cos_theta(wi))
        } else {
            0.
        }
    }
    // Used when calculating the hemispherical-directional reflectance:
    // Though, to calculate this value, we would need some samples (for some cases):
    fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>]) -> RGBSpectrum {
        // By default, performs Monte Carlo integration:
        samples
            .iter()
            .fold(RGBSpectrum::black(), |eval, &u| {
                // Sample the lobe given the sample value:
                let (result, wi, pdf) = self.sample(wo, u);
                // Only do this for non-zero pdf values (always pdf >= 0.)
                if pdf != 0. {
                    eval + result.scale(abs_cos_theta(wi) / pdf)
                } else {
                    eval
                }
            })
            // Don't forget to divide by the number of samples!
            .div_scale(samples.len() as f64)
    }
    // This performs the same calculation, but over the entire hemisphere:
    fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>]) -> RGBSpectrum {
        debug_assert!(samples0.len() == samples1.len());
        samples0
            .iter()
            .zip(samples1.iter())
            .fold(RGBSpectrum::black(), |eval, (&u0, &u1)| {
                // Use u0 to sample a wo direction:
                let wo = sample_uniform_hemisphere(u0);
                let pdfo = pdf_uniform_hemisphere();
                let (result, wi, pdfi) = self.sample(wo, u1);
                if pdfi != 0. {
                    eval + result.scale(abs_cos_theta(wi) * abs_cos_theta(wo) / (pdfo * pdfi))
                } else {
                    eval
                }
            })
            // We already checked that samples0.len() == samples1.len()
            .div_scale(f64::PI * samples0.len() as f64)
    }
}

// These functions assume one is currently in the shading space (that is, the normal is
// {0, 0, 1}).

// Returns whether or not two rays are in the same hemisphere in
// shading space:
fn is_in_same_hemisphere(w: Vec3<f64>, wp: Vec3<f64>) -> bool {
    w.z * wp.z > 0.
}

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
