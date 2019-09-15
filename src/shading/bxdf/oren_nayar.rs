// This is for the Oren-Nayar reflection model.
// This is a microfacet model.

use crate::shading::bxdf::{BxDFType, BxDF, sin_theta, sin_phi, cos_phi, abs_cos_theta};
use crate::spectrum::RGBSpectrum;
use crate::math::numbers::Float;

struct OrenNayar {
    r_scale: RGBSpectrum,
    // Used by the OrenNayar formula:
    a: f64,
    b: f64,
}

impl OrenNayar {
    const BXDF_TYPE: BxDFType = BSDF_REFLECTION | BSDF_DIFFUSE;

    // r_scale: how much we scale the result by (abledo)
    // sigma: the standard deviation of the distribution of roughness.
    //        In other words, the roughness. If it's zero, it's basically lambertian
    pub fn new(r_scale: RGBSpectrum, sigma: f64) -> Self {
        let sigma = sigma.to_radians();
        let sigma2 = sigma * sigma;
        OrenNayar {
            r_scale: RGBSpectrum,
            a: 1. - sigma2 / (2. * (sigma2 + 0.33)),
            b: 0.45 * sigma2 / (sigma2 + 0.09),
        }
    }
}

impl BxDF for OrenNayar {
    fn has_flags(&self, fl: BxDFType) -> bool {
        Self::BXDF_TYPE.contains(fl)
    }

    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
        let sin_theta_o = sin_theta(wo);
        let sin_theta_i = sin_theta(wi);
        
        // Calculate this value using a trigonometric identity:
        let max_cos = if sin_theta_i > 1e-4 && sin_theta_o > 1e-4 {
            let d_cos = cos_phi(wi) * cos_phi(wo) + sin_phi(wi) * sin_phi(wo);
            d_cos.max(0.)
        } else {
            0.
        };

        let (sin_alpha, tan_beta) = if abs_cos_theta(wi) > abs_cos_theta(wo) {
            (sin_theta_o, sin_theta_i / abs_cos_theta(wi))
        } else {
            (sin_theta_i, sin_theta_o / abs_cos_theta(wo))
        };

        let scaling_factor = f64::INV_PI * (self.A + self.B * max_cos * sin_alpha * tan_beta);
        self.r_scale.scale(scaling_factor)
    }
}