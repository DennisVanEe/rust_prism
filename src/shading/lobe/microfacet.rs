// use crate::math::numbers::Float;
// use crate::math::vector::Vec3;
// use crate::shading::bxdf::specular::Fresnel;
// use crate::shading::bxdf::{
//     abs_cos_theta, cos2_phi, cos2_theta, sin2_phi, tan2_theta, tan_theta, BxDF, BxDFType,
// };
// use crate::spectrum::RGBSpectrum;

// // Converts the roughness values between 0 and 1 to alpha values:
// fn roughness_to_alpha(roughness: f64) -> f64 {
//     let roughness = roughness.max(1e-3);
//     let x = roughness.ln();
//     1.62142 + 0.819955 * x + 0.1734 * x * x + 0.0171201 * x * x * x + 0.000640711 * x * x * x * x
// }

// // Defines a distribution for a microfacet surface.
// pub trait MicrofacetDistribution {
//     // This is the normal distribution function. That is, given a normal direction,
//     // it returns the likeness of such a normal occuring.
//     fn d(&self, wh: Vec3<f64>) -> f64;
//     // lambda: the ratio between the invisible masked microfacet area and the visible microfacet area
//     // when looking in direction w.
//     fn lambda(&self, w: Vec3<f64>) -> f64;
//     // This is Smith's masking-shadowing function. It returns the fraction
//     // of microfacets visible with normal wh when looking from direction w.
//     fn g(&self, w: Vec3<f64>, wh: Vec3<f64>) -> f64 {
//         1. / (1. + self.lambda(w) + self.lambda(wh))
//     }
//     // This is Smith's masking-shadowing function, but it only takes into
//     // account the direction we are looking from:
//     fn g1(&self, w: Vec3<f64>) -> f64 {
//         1. / (1. + self.lambda(w))
//     }
// }

// // The Beckmann Distribution
// pub struct Beckmann {
//     alpha_x: f64,
//     alpha_y: f64,
// }

// impl Beckmann {
//     // As the Beckmann distribution is anisotropic, we need to define the roughness
//     // at the major axis of the elipsoid:
//     pub fn new(roughness_x: f64, roughness_y: f64) -> Self {
//         Beckmann {
//             alpha_x: roughness_to_alpha(roughness_x),
//             alpha_y: roughness_to_alpha(roughness_y),
//         }
//     }
// }

// impl MicrofacetDistribution for Beckmann {
//     fn d(&self, wh: Vec3<f64>) -> f64 {
//         let tan2_theta = tan2_theta(wh);
//         // Check if wh "grazes" the surface
//         if tan2_theta.is_infinite() {
//             return 0.;
//         }
//         let cos4_theta = cos2_theta(wh) * cos2_theta(wh);

//         (-tan2_theta
//             * (cos2_phi(wh) / (self.alpha_x * self.alpha_x)
//                 + sin2_phi(wh) / (self.alpha_y * self.alpha_y)))
//             .exp()
//             / (f64::PI * self.alpha_x * self.alpha_y * cos4_theta)
//     }

//     fn lambda(&self, w: Vec3<f64>) -> f64 {
//         // Polynomial estimation of the actual formula:
//         let abs_tan_theta = tan_theta(w).abs();
//         if abs_tan_theta.is_infinite() {
//             return 0.;
//         }
//         // A new alpha value for the direction w (using alpha_x and alpha_y):
//         let alpha = (cos2_phi(w) * self.alpha_x * self.alpha_x
//             + sin2_phi(w) * self.alpha_y * self.alpha_y)
//             .sqrt();
//         let a = 1. / (alpha * abs_tan_theta);
//         if a >= 1.6 {
//             return 0.;
//         }
//         (1. - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
//     }
// }

// pub struct TrowbridgeReitz {
//     alpha_x: f64,
//     alpha_y: f64,
// }

// impl TrowbridgeReitz {
//     // As the Beckmann distribution is anisotropic, we need to define the roughness
//     // at the major axis of the elipsoid:
//     pub fn new(roughness_x: f64, roughness_y: f64) -> Self {
//         TrowbridgeReitz {
//             alpha_x: roughness_to_alpha(roughness_x),
//             alpha_y: roughness_to_alpha(roughness_y),
//         }
//     }
// }

// impl MicrofacetDistribution for TrowbridgeReitz {
//     fn d(&self, wh: Vec3<f64>) -> f64 {
//         let tan2_theta = tan2_theta(wh);
//         // Check if wh "grazes" the surface
//         if tan2_theta.is_infinite() {
//             return 0.;
//         }
//         let cos4_theta = cos2_theta(wh) * cos2_theta(wh);
//         let e = (cos2_phi(wh) / (self.alpha_x * self.alpha_x)
//             + sin2_phi(wh) / (self.alpha_y * self.alpha_y))
//             * tan2_theta;
//         1. / (f64::PI * self.alpha_x * self.alpha_y * cos4_theta * (1. + e) * (1. + e))
//     }

//     fn lambda(&self, w: Vec3<f64>) -> f64 {
//         let abs_tan_theta = tan_theta(w).abs();
//         if abs_tan_theta.is_infinite() {
//             return 0.;
//         }
//         // Calculate the alpha value like we did with Beckmann:
//         let alpha = (cos2_phi(w) * self.alpha_x * self.alpha_x
//             + sin2_phi(w) * self.alpha_y * self.alpha_y)
//             .sqrt();
//         let alpha2_tan2_theta = (alpha * abs_tan_theta) * (alpha * abs_tan_theta);
//         (-1. + (1. + alpha2_tan2_theta).sqrt()) / 2.
//     }
// }

// // The bottom two microfacet models use the microfacet distribution above
// // and uses the Torrance–Sparrow model.

// struct MicrofacetReflection<F: Fresnel, M: MicrofacetDistribution> {
//     microfacet: M, // the distribution
//     fresnel: F,    // the fresnel reflection
//     r_scale: RGBSpectrum,
// }

// impl<F: Fresnel, M: MicrofacetDistribution> MicrofacetReflection<F, M> {
//     pub fn new(r_scale: RGBSpectrum, microfacet: M, fresnel: F) -> Self {
//         MicrofacetReflection {
//             microfacet,
//             fresnel,
//             r_scale,
//         }
//     }
// }

// impl<F: Fresnel, M: MicrofacetDistribution> BxDF for MicrofacetReflection<F, M> {
//     const BXDF_TYPE: BxDFType = BxDFType::REFLECTION | BxDFType::GLOSSY;

//     // Evaluate the Torrance-Sparrow BRDF here:
//     fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
//         let cos_theta_o = abs_cos_theta(wo);
//         let cos_theta_i = abs_cos_theta(wi);
//         // Evaluate edge cases when grazing the surface:
//         if cos_theta_i == 0. || cos_theta_o == 0. {
//             return RGBSpectrum::black();
//         }
//         // Calculate the half-angle vector:
//         let wh = wi + wo;
//         if wh == Vec3::zero() {
//             return RGBSpectrum::black();
//         }
//         let wh = wh.normalize();
//         // Evaluate the fresnel value and microfacet distribution:
//         let f = self.fresnel.eval(wi.dot(wh));
//         // Number of microfacets that are visible:
//         let visible = self.microfacet.d(wh) * self.microfacet.g(wo, wi);
//         // The final result:
//         (self.r_scale * f)
//             .scale(visible)
//             .div_scale(4. * cos_theta_o * cos_theta_i)
//     }
// }
