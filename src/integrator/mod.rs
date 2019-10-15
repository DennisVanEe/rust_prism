pub mod direct_light;

use crate::scene::Scene;

// Each thread in PRISM gets their own integrator.

pub trait Integrator {
    fn render(&mut self, scene: &Scene);
}

// Specifies how to sample lights (either sample all lights (splitting) or sample
// just one light):
pub enum LightSamplingTechnique {
    ALL,
    ONE,
}
