pub mod area;
pub mod light_picker;
pub mod many_lights;
pub mod point;

use crate::light::many_lights::LightBound;
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
    /// varies over time, and a random value used to sample the light.
    ///
    /// Returns values in this order:
    /// *`Spectrum`: potential (if no occlusion occurs) energy the light contributes
    /// *`Vec3<f64>`: world space location of where the light will get hit (so one can calculate the wi value themselves)
    /// *`f64`: the probability density for the light sample
    fn sample(&self, point: Vec3<f64>, time: f64, u: Vec2<f64>) -> (Color, Vec3<f64>, f64);

    fn pdf(&self, point: Vec3<f64>, wi: Vec3<f64>) -> f64;

    /// Returns the total power of the light.
    fn power(&self) -> Color;

    // Whether or not the light is a delta (like a point light):
    fn is_delta(&self) -> bool;

    // Returns the normal extent and emission extent of the light in the form of a light cone.
    fn get_bound(&self) -> LightBound;

    // Returns the centroid of the light source:
    fn get_centroid(&self) -> Vec3<f64>;
}
