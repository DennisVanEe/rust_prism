pub mod direct_light;

use crate::geometry::Interaction;
use crate::light::Light;
use crate::math::ray::Ray;
use crate::math::vector::Vec2;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::shading::lobe::LobeType;
use crate::shading::material::Bsdf;
use crate::spectrum::Spectrum;

use std::f64;

// These are parameters provided to every sampler
// that are exposed to the public (that is, a user defined
// setting):
pub struct SamplerParam {
    num_pixel_samples: usize,
    num_dim: usize,
    seed: u64,
}

pub trait Integrator {
    fn setup(&mut self, scene: &Scene);
    fn render(&mut self, scene: &Scene);
}

// Calculates the balance heurisitc for the first distribution provided out of the two:
fn balance_heuristic(num_samples: usize, pdf: f64, num_samples_o: usize, pdf_o: f64) -> f64 {
    let num_samples = num_samples as f64;
    let num_samples_o = num_samples_o as f64;
    (num_samples * pdf) / (num_samples * pdf + num_samples_o + pdf_o)
}

// Calculates the power heurisitc for the first distribution provided out of the two.
// Here, the power (beta) is 2:
fn power2_heuristic(num_samples: usize, pdf: f64, num_samples_o: usize, pdf_o: f64) -> f64 {
    let num_samples = num_samples as f64;
    let num_samples_o = num_samples_o as f64;
    let s = num_samples * pdf;
    let s_o = num_samples_o * pdf_o;
    (s * s) / (s * s + s_o * s_o)
}

// Call this if you only have a single random value to sample:
fn estimate_direct<S: Sampler>(
    // Geometric information of our current position:
    int: Interaction,
    // The material at the point we are currently at (needed for MIS)
    bsdf: &Bsdf,
    // Current time we care about:
    curr_time: f64,
    // The current scene that the light belongs to (for shadow ray testing):
    scene: &Scene,

    light_sample: Vec2<f64>,
    bsdf_sample: Vec2<f64>,
    light: &dyn Light,
) -> Spectrum {
    let (light_result, light_pos, light_pdf) = light.sample(int.p, curr_time, light_sample);
    // wi points away from the surface and is normalized:
    let wi = (light_pos - int.p).normalize();
    // Now we check whether or not it's occluded:
    if scene.intersect_test(
        Ray {
            org: int.p,
            dir: wi,
        },
        f64::INFINITY,
        curr_time,
    ) {
        return Spectrum::black();
    }

    // Calculate how much the light contributes in this case. Sampling it only once.
    let light_contrib = if light_pdf > 0. && !light_result.is_black() {
        // TODO: figure out what lobe flags we should use here:
        // Evaulate the bsdf at the surface:
        let bsdf_result = bsdf
            .eval(int.wo, wi, LobeType::ALL)
            .scale(wi.dot(int.shading_n).abs());
        let bsdf_pdf = bsdf.pdf(int.wo, wi, LobeType::ALL);

        // Check if the light is a "delta light". This is a special case that
        // always returns 1 for the pdf. If that is the case, we don't have to
        // worry about MIS:
        if light.is_delta() {
            // Normal monte carlo estimator:
            bsdf_result * light_result.div_scale(light_pdf)
        } else {
            // MIS monte carlo estimator:
            let mis_w = power2_heuristic(1, light_pdf, 1, bsdf_pdf);
            (bsdf_result * light_result).scale(mis_w / light_pdf)
        }
    } else {
        Spectrum::black()
    };

    // Now we see how much the bsdf contributes:
    let bsdf_contrib = if !light.is_delta() {
        // Sample the bsdf (TODO: figure out the LobeType flag).
        // The bsdf only returns None for the lobe type if none of the types of lobes
        // we are sampling match it:
        let (bsdf_result, bsdf_wi, bsdf_pdf, lobe_type) =
            bsdf.sample(int.wo, bsdf_sample, LobeType::ALL);
        if !bsdf_result.is_black() && bsdf_pdf > 0. {
            // Project it:
            let bsdf_result = bsdf_result.scale(bsdf_wi.dot(int.shading_n).abs());
            // Only bother sampling the light if the lobe isn't specular. If it is,
            // it's unlikely we will hit it:
            if !lobe_type.contains(LobeType::SPECULAR) {
                // TODO: figure out how to handle the case of area lights. I think I'll do what
                // pbrt does and have some meshes with an attached light. This should allow for emissive
                // geometry and whatnot. I'll see how much I can control it (might add support for textures and
                // whatnot to the light):
            }
        }
    } else {
        // If it's a delta distribution, then don't bother contributing
        // anything from the bsdf as there is no way we'll hit it:
        Spectrum::black()
    };
}

// Some important functions that may be useful for all integrators:

// This is an integrator that uniformly samples all lights in a scene:
fn uniform_sample_all_lights<S: Sampler>(
    // Point from which we are sampling:
    int: Interaction,
    // All of the lights in the scene:
    lights: &[&dyn Light],
    // The Sampler we are using to sample values:
    sampler: &mut S,
) -> Spectrum {
    // Loop over all the lights in the scene here:
    lights.iter().fold(Spectrum::black(), |total, &curr_light| {
        // Don't worry about scattering media for now:
        let light_samples = sampler.get_2d_array();
        if light_samples.is_empty() {
            total + estimate_direct()
        } else {
            let sum_samples = light_samples
                .iter()
                .fold(Spectrum::black(), |total, &curr_sample| {
                    total + estimate_direct()
                });
            total + (sum_samples.div_scale(light_samples.len() as f64))
        }
    })
}

fn uniform_sample_one_light<S: Sampler>(
    // Point from which we are sampling:
    int: Interaction,
    // All of the lights in the scene:
    lights: &[&dyn Light],
    // The Sampler we are using to sample values:
    sampler: &mut S,
) -> Sepctrum {
    // Check if we have any lights in the scene at all:
    if lights.is_empty() {
        return Spectrum::black();
    }

    let num_lights = lights.len() as f64;

    // Randomly pick a light:
    let light = {
        let sample = sampler.get_1d();
        let light_index = (lights.len() - 1).min((num_lights * sample) as usize);
        lights[light_index]
    };

    let light_sample = sampler.get_2d();
    estimate_direct().scale(num_lights)
}
