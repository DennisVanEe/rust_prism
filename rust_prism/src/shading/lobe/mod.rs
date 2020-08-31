pub mod lambertian;
//pub mod microfacet;
//pub mod oren_nayar;
//pub mod specular;

use crate::geometry::GeomInteraction;
use crate::spectrum::Color;
use bitflags::bitflags;
use num_traits::clamp;
use pmath::sampling;
use pmath::vector::{Vec2, Vec3};

bitflags! {
    pub struct LobeType : u32 {
        const REFLECTION = 1 << 0;
        const TRANSMISSION = 1 << 1;
        const DIFFUSE = 1 << 2;
        const GLOSSY = 1 << 3;
        const SPECULAR = 1 << 4;
        // Useful for certain situations:
        const NONE = 1 << 5;
        const ALL = Self::REFLECTION.bits |
            Self::TRANSMISSION.bits | Self::DIFFUSE.bits |
            Self::GLOSSY.bits | Self::SPECULAR.bits;
    }
}

/// This is a trait that represents brdf (reflections) and btdf (transmissions).
pub trait Lobe {
    /// Returns whether or not the lobe has these types present.
    /// This will be redundant as hell, but rust does not support fields
    /// in traits.
    fn contains_type(&self, lobe_type: LobeType) -> bool;
    /// Returns the lobe type:
    fn get_type(&self) -> LobeType;
    /// Evaluates the lobe (wo and wi are in shading space). It also includes an interaction
    /// in case the lobe needed extra information with regards to it.
    fn eval(&self, wo: Vec3<f64>, wi: Vec3<f64>, interaction: GeomInteraction) -> Color;
    /// Sampling the lobe and also works when we have a delta function
    /// (for instance, with perfectly specular surfaces). Note that wo is in shading space.
    /// If the trait isn't implemented, it uses a cosine hemisphere sampling technique.
    fn sample(
        &self,
        wo: Vec3<f64>,
        u: Vec2<f64>,
        interaction: GeomInteraction,
    ) -> (Color, Vec3<f64>, f64) {
        // If wo.z < 0 then it's not on the side of the normal. Because we are sampling
        // a hemisphere in the shading space, we need to flip around the final z result
        // to make sure it's on the same side as wo:
        let wi = if wo.z < 0. {
            let p = sampling::cos_sample_hemisphere(u);
            Vec3 {
                x: p.x,
                y: p.y,
                z: -p.z, // flip around
            }
        } else {
            sampling::cos_sample_hemisphere(u)
        };
        (self.eval(wo, wi), wi, self.pdf(wo, wi))
    }
    /// Returns the pdf of distribution used to sample the lobe given the incoming and
    /// the outgoing directions. Both of which are in shading space and point away from
    /// the surface.
    /// If the trait isn't implemented, it assumes a cosine weighted hemisphere.
    fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>, interaction: GeomInteraction) -> f64 {
        if is_in_same_hemisphere(wo, wi) {
            sampling::cos_sphere_pdf(abs_cos_theta(wi))
        } else {
            0.
        }
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
