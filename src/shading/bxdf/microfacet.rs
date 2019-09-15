use crate::math::numbers::Float;
use crate::math::vector::Vec3;
use crate::shading::bxdf::{cos2_phi, cos2_theta, sin2_phi, tan2_theta};

// Converts the roughness values between 0 and 1 to alpha values:
pub fn roughness_to_alpha(roughness: f64) -> f64 {
    let roughness = roughness.max(1e-3);
    let x = roughness.ln();
    1.62142 + 0.819955 * x + 0.1734 * x * x + 0.0171201 * x * x * x + 0.000640711 * x * x * x * x
}

// Defines a distribution that defines a microfacet surface.
pub trait MicrofacetDistribution {
    // This is the normal distribution function. That is, given a normal direction,
    // it returns the likeness of such a normal occuring.
    fn normal_dist(&self, wh: Vec3<f64>) -> f64;
    // This is Smith's masking-shadowing function (G_1). It returns the fraction
    // of microfacets visible with normal wh when looking from direction w.
    fn mask_shadow_dist(&self, w: Vec3<f64>, wh: Vec3<f64>) -> f64;
}

// The Beckmann Distribution
pub struct Beckmann {
    alpha_x: f64,
    alpha_y: f64,
}

impl Beckmann {
    // As the Beckmann distribution is anisotropic, we need to define the roughness
    // at the major axis of the elipsoid:
    pub fn new(roughness_x: f64, roughness_y: f64) -> Self {
        Beckmann { 
            alpha_x: roughness_to_alpha(roughness_x),
            alpha_y: roughness_to_alpha(roughness_y), 
        }
    }
}

impl MicrofacetDistribution for Beckmann {
    fn normal_dist(&self, wh: Vec3<f64>) -> f64 {
        let tan2_theta = tan2_theta(wh);
        // Check if wh "grazes" the surface
        if tan2_theta.is_infinite() {
            return 0.;
        }
        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);

        (-tan2_theta
            * (cos2_phi(wh) / (self.alpha_x * self.alpha_x)
                + sin2_phi(wh) / (self.alpha_y * self.alpha_y)))
            .exp()
            / (f64::PI * self.alpha_x * self.alpha_y * cos4_theta)
    }
}

pub struct TrowbridgeReitz {
    alpha_x: f64,
    alpha_y: f64,
}

impl TrowbridgeReitz {
    // As the Beckmann distribution is anisotropic, we need to define the roughness
    // at the major axis of the elipsoid:
    pub fn new(roughness_x: f64, roughness_y: f64) -> Self {
        TrowbridgeReitz { 
            alpha_x: roughness_to_alpha(roughness_x),
            alpha_y: roughness_to_alpha(roughness_y), 
        }
    }
}

impl MicrofacetDistribution for TrowbridgeReitz {
    fn normal_dist(&self, wh: Vec3<f64>) -> f64 {
        let tan2_theta = tan2_theta(wh);
        // Check if wh "grazes" the surface
        if tan2_theta.is_infinite() {
            return 0.;
        }
        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);
        let e = (cos2_phi(wh) / (self.alpha_x * self.alpha_x) + sin2_phi(wh) / (self.alpha_y * self.alpha_y)) * tan2_theta;
        1. / (f64::PI * self.alpha_x * self.alpha_y * cos4_theta * (1. + e) * (1. + e))
    }
}
