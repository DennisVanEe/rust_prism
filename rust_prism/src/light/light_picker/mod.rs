pub mod uniform_all;
pub mod uniform_one;

use crate::geometry::GeomInteraction;
use crate::light;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::shading::material::Bsdf;
use crate::spectrum::Color;
use pmath::vector::Vec3;

/// Generates an iterator to iterate over all of the lights that were chosen.
pub trait LightPicker<I: Iterator<Item = (usize, f64)>> {
    /// All lights in the scene are described using a Light ID starting from 0 to `num_lights` (exclusive).
    /// If any allocation is required, make sure to do that in this step.
    fn set_scene_lights(&mut self, num_lights: usize, scene: &Scene);

    /// Picks a number of lights and returns an iterator to those lights.
    fn pick_lights(
        &self,
        shading_point: Vec3<f64>,
        normal: Vec3<f64>,
        sampler: &mut Sampler,
        scene: &Scene,
    ) -> I;
}

/// Samples all of the lights in a scene given a light picker.
pub fn sample_lights<I: Iterator<Item = (usize, f64)>, L: LightPicker<I>>(
    interaction: GeomInteraction,
    bsdf: &Bsdf,
    time: f64,
    scene: &Scene,
    sampler: &mut Sampler,
    light_picker: &L,
) -> Color {
    let light_iter = light_picker.pick_lights(interaction.p, interaction.shading_n, sampler, scene);
    let mut final_color = Color::black();
    for (light_id, light_scale) in light_iter {
        // TODO: explore whether to make specular false.
        final_color = final_color
            + light::estimate_direct_light(
                interaction,
                bsdf,
                time,
                sampler,
                scene,
                light_id,
                false,
            )
            .scale(light_scale);
    }

    final_color
}
