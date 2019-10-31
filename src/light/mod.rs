pub mod point;

use crate::math::vector::{Vec2, Vec3};
use crate::spectrum::RGBSpectrum;

use bitflags::bitflags;

bitflags! {
    pub struct LightType : u32 {
        // Whether or not the light is a delta position light (that is, it's position
        // is a delta function):
        const DELTA_POSITION = 1 << 0;
        // Whether the direction is a delta direction
        const DELTA_DIRECTION = 1 << 1;
        const AREA = 1 << 2;
        const INFINITE = 1 << 3;
    }
}

// The light interface:
pub trait Light {
    // Samples the light at a given interaction point (in scene space):
    // u is a random uniform point (useful for things like area lights).
    // The time value is not for handling moving lights (that's already handled elsewhere).
    // No, it's for lights that have some special time-dependent effect.
    //
    // Returns values in this order:
    // pdf: the probability density for the light sample
    // RGBSpectrum: potential (if no occlusion occurs) energy the light contributes
    // Vec3<f64>: scene space location of where the light will get hit (so one can calculate the wi value themselves)
    fn sample(
        &self,
        surface_point: Vec3<f64>,
        time: f64,
        u: Vec2<f64>,
    ) -> (RGBSpectrum, Vec3<f64>, f64);

    fn pdf(&self, surface_point: Vec3<f64>, wi: Vec3<f64>) -> f64;

    fn power(&self) -> RGBSpectrum;

    fn num_recommended_samples(&self) -> usize {
        
    }
}

// This essentially calculates Planck's law for a range of wavelengths.
// NOTE: the wavelengths must be in terms of nm.
pub fn blackbody(wavelengths: &[f64], temp: f64, spd: &mut [f64]) {
    // Some important constant values when calculating this value:
    const c: f64 = 299792458.;
    // Planck constant:
    const h: f64 = 6.62606957e-34;
    // Boltzmann constant:
    const kb: f64 = 1.3806488e-23;
    for (&wl, le) in wavelengths.iter().zip(spd.iter_mut()) {
        // convert nm -> m
        let wl = wl * 1e-9;
        // wl^5:
        let wl5 = (wl * wl) * (wl * wl) * wl;
        *le = (2. * h * c * c) / (wl5 * ((h * c) / (wl * kb * temp)).exp() - 1.);
    }
}

// This is the blackbody but normalized (max value in SPD is 1):
pub fn blackbody_normalized(wavelengths: &[f64], temp: f64, spd: &mut [f64]) {
    // First we call the regular blackbody function:
    blackbody(wavelengths, temp, spd);
    // Use Wein's displacement law to calculate the wavelength with the maximum emssision:
    let wavelength_max = [2.8977721e-3 / temp * 1e9];
    let mut max_emission = [0.; 1];
    blackbody(&wavelength_max, temp, &mut max_emission);
    // Finally we can go ahead and normalize the result:
    for v in spd.iter_mut() {
        *v /= max_emission[0];
    }
}
