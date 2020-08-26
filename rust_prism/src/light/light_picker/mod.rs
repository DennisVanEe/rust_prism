pub mod uniform_all;
pub mod uniform_one;

use crate::light::many_lights::LightBound;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Color;
use pmath::vector::{Vec2, Vec3};

/// Given a random number, returns a number of lights to sample.
pub trait LightPicker {
    /// All lights in the scene are described using a Light ID starting from 0 to `num_lights` (exclusive).
    /// If any allocation is required, make sure to do that in this step.
    fn set_scene_lights(&mut self, num_lights: usize, scene: &Scene);

    /// Runs through the process of picking the needed lights. Call this before calling `get_next_light`.
    /// It returns the random number rescaled so that it may be used again later.
    fn pick_lights(
        &mut self,
        shading_point: Vec3<f64>,
        normal: Vec3<f64>,
        sampler: &mut Sampler,
        scene: &Scene,
    );

    /// Loops over all of the lights until None is returned, indicating no more lights need to get sampled.
    /// It also returns a weight to apply to the specific light to ensure correct
    fn get_next_light(&mut self) -> Option<(usize, f64)>;
}
