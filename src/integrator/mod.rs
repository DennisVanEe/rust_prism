pub mod direct_light;

use crate::scene::Scene;

// Each thread in PRISM gets their own integrator.

pub trait Integrator {
    fn setup(&mut self, scene: &Scene);
    fn render(&mut self, scene: &Scene);
}
