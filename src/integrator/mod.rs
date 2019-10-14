pub mod direct_light;

use crate::scene::Scene;

pub trait Integrator {
    fn render(&mut self, scene: &Scene);
}