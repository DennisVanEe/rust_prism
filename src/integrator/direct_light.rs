use super::{Integrator, SamplerParam};

use crate::sampler::Sampler;
use crate::scene::Scene;

// Specifies how to sample lights (either sample all lights (splitting) or sample
// just one light):
pub enum LightTechnique {
    // Sample all of the lights in the scene
    ALL,
    // Only sample a single light
    ONE,
}

// A very basic integrator that only takes into account direct light and nothing else.
// Not really the most exciting thing in the world. Useful for quick testing of things
// though.
pub struct DirectLight<S: Sampler> {
    sampler: S,
    // Specifies how we want to sample the lights (we can either
    // sample all of them or just randomly sample some of them):
    light_technique: LightTechnique,
    // Specifies the maximum depth of the integrator:
    max_depth: u32,
}

impl<S: Sampler> DirectLight<S> {
    pub fn new(
        // Extra sampler parameters that might be needed:
        ex_sampler_param: S::ParamType,
        // Default sampler parameters:
        def_sampler_param: SamplerParam,
        light_technique: LightTechnique,
        max_depth: u32,
    ) -> Self {
        if self.light_technique {}

        Self {
            sampler: S::new(
                ex_sampler_param,
                def_sampler_param.num_pixel_samples,
                def_sampler_param.num_dim,
                def_sampler_param.seed,
            ),
            light_technique,
            max_depth,
        }
    }
}

impl<S: Sampler> Integrator for DirectLight<S> {
    fn setup(&mut self, scene: &Scene) {
        if self.light_technique {}
    }

    fn render(&mut self, scene: &Scene) {}
}
