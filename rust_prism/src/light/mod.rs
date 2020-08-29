pub mod area;
pub mod light_picker;
pub mod many_lights;
pub mod point;

use crate::light::many_lights::LightBound;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Color;
use pmath::vector::{Vec2, Vec3};

use bitflags::bitflags;

bitflags! {
    pub struct LightType : u32 {
        // Whether or not the light is a delta position light (that is, it's position
        // is a delta function):
        const DELTA_POSITION = 1 << 0;
        // Whether the direction is a delta direction
        const DELTA_DIRECTION = 1 << 1;
        const AREA = 1 << 2;
        const INFINITE = 1 << 3;
    }
}

/// An interface for defining a light in the scene. Lights are transformed into world
/// space when being committed to a scene.
pub trait Light: Sync + 'static {
    /// Samples the light from a specific position (`point`) in world space, a `time` in case the light
    /// varies over time, the `scene` in case it needs it, and a random value (`u`) used to sample the light.
    ///
    /// Returns values in this order:
    /// *`Color`: potential (if no occlusion occurs) energy the light contributes
    /// *`Vec3<f64>`: world space location of where the light will get hit (so one can calculate the wi value themselves)
    /// *`f64`: the probability density for the light sample
    fn sample(
        &self,
        point: Vec3<f64>,
        time: f64,
        scene: &Scene,
        u: Vec2<f64>,
    ) -> (Color, Vec3<f64>, f64);

    /// Given a point and direction in light space, returns the pdf.
    fn pdf(&self, point: Vec3<f64>, wi: Vec3<f64>) -> f64;

    /// Returns the total power of the light.
    fn power(&self) -> Color;

    /// Whether or not the light is a delta (like a point light):
    fn is_delta(&self) -> bool;

    /// Returns the centroid of the light source:
    fn get_centroid(&self) -> Vec3<f64>;
}

/// Samples a light directly using MIS. If there is occlusion, false (and color is black), otherwise
/// it returns true and whatever the color is.
///
/// # Arguments
/// * `shading_point`: World space of the point where we are shading form.
/// * `time`: The time
/// * `sampler`: The sampler used to sample the bsdf and light.
/// * `scene`: The scene used for visibility testing and used by the light if necessary.
/// * `light_id`: The light id of the light we are directly sampling.
pub fn estimate_direct_light(
    shading_point: Vec3<f64>,
    time: f64,
    sampler: &mut Sampler,
    scene: &Scene,
    light_id: u32,
) -> Color {
    let light = scene.get_light(light_id);

    // First we sample the light source:
    let (light_color, light_point, light_pdf) =
        light.sample(shading_point, time, scene, sampler.sample());

    if (light_pdf > 0.0) && !light_color.is_black() {}

    todo!();
}
