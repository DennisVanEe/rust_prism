use super::{Integrator, LightSamplingTechnique};

use crate::sampler::Sampler;
use crate::scene::Scene;

// A very basic integrator that only takes into account direct light and nothing else.
// Not really the most exciting thing in the world. Useful for quick testing of things
// though.
pub struct DirectLight<S: Sampler> {
    light_sampling_technique: LightSamplingTechnique,
    // Specifies the maximum depth of the integrator:
    max_depth: u32,
}

impl DirectLight {
    pub fn new(light_sampling_technique: LightSamplingTechnique, max_depth: u32) -> Self {
        Self {
            light_sampling_technique,
            max_depth,
        }
    }
}

impl Integrator for DirectLight {
    fn render(&mut self, scene: &Scene) {}
}
