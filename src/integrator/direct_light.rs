use super::Integrator;

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
    light_technique: LightTechnique,
    // Specifies the maximum depth of the integrator:
    max_depth: u32,
}

impl<S: Sampler> DirectLight<S> {
    pub fn new(light_technique: LightTechnique, max_depth: u32) -> Self {
        Self {
            light_technique,
            max_depth,
        }
    }
}

impl<S: Sampler> Integrator for DirectLight<S> {
    fn setup(&mut self, scene: &Scene) {
        if self.light_technique {
            
        }
    }

    fn render(&mut self, scene: &Scene) {}
}
