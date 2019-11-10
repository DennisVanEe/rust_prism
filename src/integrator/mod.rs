pub mod direct_light;

use crate::geometry::Interaction;
use crate::light::Light;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;
use crate::math::vector::Vec2;
use crate::math::ray::Ray;
use crate::shading::material::Bsdf;
use crate::shading::lobe::LobeType;

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
    // Light sample and light we care about:
    light_sample: Vec2<f64>,
    light: &dyn Light) -> Spectrum {

    let (li, light_pos, light_pdf) = light.sample(int.p, curr_time, light_sample);
    // wi points away from the surface and is normalized:
    let wi = (light_pos - int.p).normalize();
    // Now we check whether or not it's occluded:
    if scene.intersect_test(Ray { org: int.p, dir: wi }, f64::INFINITY, curr_time) {
        return Spectrum::black();
    }

    if light_pdf > 0. && !li.is_black() {
        // TODO: figure out what lobe flags we should use here:
        // Evaluate rendering equation here:
        let result = bsdf.eval(int.wo, wi, LobeType::ALL).scale(wi.dot(int.shading_n));
        let pdf = bsdf.pdf(int.wo, wi, LobeType::ALL);

        // Check if the light is a "delta light". This is a special case that 
        // always returns 1 for the pdf. If that is the case, we don't have to
        // worry about MIS:
        if light.is_delta() {
            result * li.div_scale(pdf);
        } else {

        }

        Spectrum::black()
    } else {
        Spectrum::black()
    }
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
    lights.iter().fold(Spectrum::black(),
        |total, &curr_light| {
            // Don't worry about scattering media for now:
            let light_samples = sampler.get_2d_array();
            if light_samples.is_empty() {
                total + estimate_direct()
            } else {
                let sum_samples = light_samples.iter().fold(Spectrum::black(),
                    |total, &curr_sample| {
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
