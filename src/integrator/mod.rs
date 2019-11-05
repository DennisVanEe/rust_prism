pub mod direct_light;

use crate::geometry::Interaction;
use crate::light::Light;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::RGBColor;

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
// TODO: finish this
fn estimate_direct()

// Some important functions for most all integrators:

fn uniform_sample_all_lights<S: Sampler>(
    int: Interaction,
    lights: &[&dyn Light],
    sampler: &mut S,
) -> RGBColor {
    // Loop over all the lights in the scene here:
    lights.iter().fold(RGBColor::black(),
        |total, &curr_light| {
            // Don't worry about scattering media yet:
            let light_samples = sampler.get_2d_array();
            if light_samples.is_empty() {
                total + estimate_direct()
            } else {
                let sum_samples = light_samples.iter().fold(RGBColor::black(),
                    |total, &curr_sample| {
                        total + estimate_direct()
                    });
                total + (sum_samples / light_samples.len())
            }
        })
}
