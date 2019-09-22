use crate::math::util::{align, refract};
use crate::math::vector::{Vec2, Vec3};
use crate::shading::lobe::{abs_cos_theta, cos_theta, Lobe, LobeType};
use crate::spectrum::RGBSpectrum;

use num_traits::clamp;

// Computes the fresnel reflectance given the cosine of the incident angle.
pub trait Fresnel {
    fn eval(&self, cos_theta_i: f64) -> RGBSpectrum;
}

//
// For Dielectrics:
//
// Dielectrics transmit a certain portion of the light that hits it.
// They have real-valued indices of refraction
// They include: glass, water, etc.
#[derive(Clone, Copy)]
pub struct Dielectric {
    eta_i: f64,
    eta_t: f64,
}

impl Dielectric {
    pub fn new(eta_i: f64, eta_t: f64) -> Self {
        Dielectric { eta_i, eta_t }
    }
}

impl Fresnel for Dielectric {
    fn eval(&self, cos_theta_i: f64) -> RGBSpectrum {
        RGBSpectrum::from_scalar(fr_dielectric(cos_theta_i, self.eta_i, self.eta_t))
    }
}

//
// For Conductors:
//
// These don't tend to transmit much light (most of it gets absorbed pretty early on in
// the surface. So, they tend to be opaque (unless you have, like, really thin material).
// They have complex indices of refraction: n + ki, with k being the "absorbtion coefficient".
#[derive(Clone, Copy)]
pub struct Conductor {
    eta_i: RGBSpectrum,
    eta_t: RGBSpectrum,
    k: RGBSpectrum,
}

impl Conductor {
    pub fn new(eta_i: RGBSpectrum, eta_t: RGBSpectrum, k: RGBSpectrum) -> Self {
        Conductor { eta_i, eta_t, k }
    }
}

impl Fresnel for Conductor {
    fn eval(&self, cos_theta_i: f64) -> RGBSpectrum {
        fr_conductor(cos_theta_i.abs(), self.eta_i, self.eta_t, self.k)
    }
}

//
// This Fresnel
//
// This is for cases where no transmission is desired, so it always returns 1. when eval
// is called.
#[derive(Clone, Copy)]
pub struct PerfectMirror {}

impl PerfectMirror {
    pub fn new() -> Self {
        PerfectMirror {}
    }
}

impl Fresnel for PerfectMirror {
    // This will always return 1. so it perfectly reflects all light:
    fn eval(&self, cos_theta_i: f64) -> RGBSpectrum {
        RGBSpectrum::from_scalar(1.)
    }
}

// Calculates the Fresnel Reflectance of a dielectric:
//
// As we are assuming the light isn't polarized, we take the average
// of the Fresnel reflectance of the light that is parallel polarized (s-polarized)
// and perpendicular polarized (p-polarized):
//
// cos_theta_I: the cosine of the incident angle
// eta_i: index of refraction of the incident medium (whatever material we are coming from)
// eta_t: index of refraction of the transmitted medium (whatever material we are entering)
pub fn fr_dielectric(cos_theta_i: f64, eta_i: f64, eta_t: f64) -> f64 {
    let cos_theta_i = clamp(cos_theta_i, -1., 1.);
    let cos_theta_t = 5.;

    // Check if we are entering (the ray is outside of, so the cosine is positive):
    // If we are not entering, then we need to make sure we update eta_i and eta_t. In
    // other words, we "flip" our perspective:
    let (cos_theta_i, eta_i, eta_t) = if cos_theta_i < 0. {
        (cos_theta_i.abs(), eta_t, eta_i) // swap them and make cos_theta_i positive
    } else {
        (cos_theta_i, eta_i, eta_t) // they remain unchanged
    };

    // This is just the identity: cos^2 + sin^2 = 1
    let sin_theta_i = (1. - cos_theta_i * cos_theta_i).max(0.).sqrt();
    // Using Snell's law to find the sin of the transmitted angle
    let sin_theta_t = eta_i / eta_t * sin_theta_i;
    // Check for total internal reflection:
    if sin_theta_t > 1. {
        return 1.;
    }
    // Once again, apply the identity:
    let cos_theta_t = (1. - sin_theta_t * sin_theta_t).max(0.).sqrt();

    // Apply Fresnel's equations for reflectance:
    let refl_parl = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t))
        / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
    let refl_perp = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t))
        / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));

    // Average the result as we are dealing with unpolarized light
    (refl_parl * refl_parl + refl_perp * refl_perp) / 2.
}

// This calculates the fresnel reflectance given a conductor. This means
// we need to provide an absorbtion factor. If k == 0, then this acts like
// fr_dielectric.
//
// NOTE: The cos_theta_i value is measured with respect to the normal being on the
// same side as w_i (incident). That means we don't do the flip like above.
pub fn fr_conductor(
    cos_theta_i: f64,
    eta_i: RGBSpectrum,
    eta_t: RGBSpectrum,
    k: RGBSpectrum,
) -> RGBSpectrum {
    let cos_theta_i = clamp(cos_theta_i, -1., 1.);
    let eta = eta_t / eta_i;
    let eta_k = k / eta_i;

    // The identities as we have seen them before:
    let cos2_theta_i = cos_theta_i * cos_theta_i;
    let sin2_theta_i = 1. - cos2_theta_i;

    let eta2 = eta * eta;
    let eta_k2 = eta_k * eta_k;

    let t0 = eta2 - eta_k2 - RGBSpectrum::from_scalar(sin2_theta_i);
    let a2_plus_b2 = (t0 * t0 + (eta2 * eta_k2).scale(4.)).sqrt();
    let t1 = a2_plus_b2 + RGBSpectrum::from_scalar(cos2_theta_i);
    let a = (a2_plus_b2 + t0).scale(0.5).sqrt();
    let t2 = a.scale(cos_theta_i * 2.);
    let rs = (t1 - t2) / (t1 + t2);

    let t3 = a2_plus_b2.scale(cos2_theta_i) + RGBSpectrum::from_scalar(sin2_theta_i * sin2_theta_i);
    let t4 = t2.scale(sin2_theta_i);
    let rp = rs * (t3 - t4) / (t3 + t4);

    (rp + rs).scale(0.5)
}

//
// SpecularReflection
//
// Defines how light reflects from an object:

struct SpecularReflection<F: Fresnel> {
    // Defines the type of reflection:
    fresnel: F,
    // Scales the reflected color:
    r_scale: RGBSpectrum,
}

impl<F: Fresnel> SpecularReflection<F> {
    const LOBE_TYPE: LobeType = LobeType::REFLECTION | LobeType::SPECULAR;

    pub fn new(r_scale: RGBSpectrum, fresnel: F) -> Self {
        SpecularReflection { fresnel, r_scale }
    }
}

impl<F: Fresnel> Lobe for SpecularReflection<F> {
    fn has_type(&self, fl: LobeType) -> bool {
        Self::LOBE_TYPE.contains(fl)
    }

    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
        // This always returns black (even, if by some miracle, we hit the right direction
        // straight on)
        RGBSpectrum::black()
    }

    fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> f64 {
        // Just like above, this will always return 0 as we won't hit the correct angle
        0.
    }

    fn sample_f(&self, wo: Vec3<f64>, sample: Vec2<f64>) -> (RGBSpectrum, Vec3<f64>, f64) {
        // This is basically calling reflect(wo, n) with n = (0, 0, 1)
        let wi = Vec3 {
            x: -wo.x,
            y: -wo.y,
            z: wo.z,
        };
        let pdf = 1.; // We always pick this direction, so this is 1

        // This is just the case we have when using dielectrics:
        // f_r(w_o, w_i) = F_r(w_r) * (delta(w_i - w_r) / | cos_theta_r |)
        let spectrum =
            (self.fresnel.eval(cos_theta(wi)) * self.r_scale).scale(1. / abs_cos_theta(wi));
        (spectrum, wi, pdf)
    }
}

//
// SpecularTransmission
//
// Defines how light transmits through the object.
#[derive(Clone, Copy)]
pub struct SpecularTransmission {
    // Conductors don't transmit light, so we need a dielectric:
    fresnel: Dielectric,
    // Scales the reflected color:
    t_scale: RGBSpectrum,
    eta_above: f64,
    eta_below: f64,
}

impl SpecularTransmission {
    const LOBE_TYPE: LobeType = LobeType::TRANSMISSION | LobeType::SPECULAR;

    // In pbrt we define a transport mode, but we aren't using bidirectional techniques this case,
    // so we can ignore that.
    // eta_above: index of refraction above the surface we are intersecting (based on normal)
    // eta_below: index of refraction below the surface we are interescting (based on normal)
    pub fn new(t_scale: RGBSpectrum, eta_above: f64, eta_below: f64) -> Self {
        SpecularTransmission {
            fresnel: Dielectric::new(eta_above, eta_below),
            t_scale,
            eta_above,
            eta_below,
        }
    }
}

impl Lobe for SpecularTransmission {
    fn has_type(&self, fl: LobeType) -> bool {
        Self::LOBE_TYPE.contains(fl)
    }

    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
        // See SpecularReflection:
        RGBSpectrum::black()
    }

    fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> f64 {
        // See SpecularReflection:
        0.
    }

    fn sample_f(&self, wo: Vec3<f64>, sample: Vec2<f64>) -> (RGBSpectrum, Vec3<f64>, f64) {
        // Pick the correct eta_i and eta_t depending on the directin of w_o compared to the
        // normal:
        let (eta_i, eta_t) = if cos_theta(wo) > 0. {
            (self.eta_above, self.eta_below)
        } else {
            (self.eta_below, self.eta_above)
        };

        // We need w_i (the outgoing ray). Just call refract for this (similar to glsl's refract function).
        let wi = match refract(
            wo,
            align(
                Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 1.,
                },
                wo,
            ),
            eta_i / eta_t,
        ) {
            Some(w) => w,
            None => return (RGBSpectrum::black(), Vec3::zero(), 1.),
        };

        let pdf = 1.;
        let spectrum = (self.t_scale
            * (RGBSpectrum::from_scalar(1.) - self.fresnel.eval(cos_theta(wi))))
        .scale((eta_i * eta_i) / (eta_t * eta_t * abs_cos_theta(wi)));
        (spectrum, wi, pdf)
    }
}

//
// FresnelSpecular
//
// Combines both Specular Reflection Specular Transmission as opposed to
// just one of them like the previous ones.
#[derive(Clone, Copy)]
pub struct SpecularFresnal {
    // Again, we only focus on the dielectric case (conductors have no transmission):
    fresnel: Dielectric,
    // Scales for transmission:
    t_scale: RGBSpectrum,
    // Scales for reflection:
    r_scale: RGBSpectrum,
    eta_above: f64,
    eta_below: f64,
}

impl SpecularFresnal {
    const LOBE_TYPE: LobeType = LobeType::REFLECTION | LobeType::TRANSMISSION | LobeType::SPECULAR;

    // In pbrt we define a transport mode, but we aren't using bidirectional techniques this case,
    // so we can ignore that.
    // refl_scale: the scaling factor for the reflected portion
    // trans_scale: the scaling factor for the transmitted portion
    // eta_above: index of refraction above the surface we are intersecting (based on normal)
    // eta_below: index of refraction below the surface we are interescting (based on normal)
    pub fn new(t_scale: RGBSpectrum, r_scale: RGBSpectrum, eta_above: f64, eta_below: f64) -> Self {
        SpecularFresnal {
            fresnel: Dielectric::new(eta_above, eta_below),
            t_scale,
            r_scale,
            eta_above,
            eta_below,
        }
    }
}

impl Lobe for SpecularFresnal {
    fn has_type(&self, fl: LobeType) -> bool {
        Self::LOBE_TYPE.contains(fl)
    }

    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
        // See SpecularReflection:
        RGBSpectrum::black()
    }

    fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> f64 {
        // See SpecularReflection:
        0.
    }
}
