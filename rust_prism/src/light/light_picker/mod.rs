pub mod uniform_all;
pub mod uniform_one;

use crate::geometry::GeomInteraction;
use crate::light;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::shading::material::Bsdf;
use crate::spectrum::Color;
use pmath::vector::Vec3;

/// A `LightPickerManager` is used to spawn light pickers for each thread and maintain any
/// information that a light picker may need across different threads may want to use. It is guaranteed
/// that the LightPickerManager instance will exist until all threads have finished rendering.
pub trait LightPickerManager<L: LightPicker>: Sync {
    /// Spawns an integrator for a specific thread with the provided id.
    fn spawn_lightpicker(&self, thread_id: u32) -> L;
}

/// Given a random number, returns a number of lights to sample.
pub trait LightPicker {
    /// All lights in the scene are described using a Light ID starting from 0 to `num_lights` (exclusive).
    /// If any allocation is required, make sure to do that in this step.
    fn set_scene_lights(&mut self, num_lights: usize, scene: &Scene);

    /// Runs through the process of picking the needed lights. Call this before calling `get_next_light`.
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

/// Samples all of the lights in a scene given a light picker.
pub fn sample_lights(
    interaction: GeomInteraction,
    bsdf: &Bsdf,
    time: f64,
    scene: &Scene,
    sampler: &mut Sampler,
    light_picker: &mut dyn LightPicker,
) -> Color {
    light_picker.pick_lights(interaction.p, interaction.shading_n, sampler, scene);
    let mut final_color = Color::black();
    while let Some((light_id, light_scale)) = light_picker.get_next_light() {
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
