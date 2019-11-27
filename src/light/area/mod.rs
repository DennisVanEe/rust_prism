// An area light is basically a special type of light that takes the form of some
// sort of geometry:

pub mod diffuse;

use super::Light;
use crate::geometry::{GeomInteraction, Geometry};
use crate::math::vector::Vec3;
use crate::spectrum::Spectrum;

// An area light is a special type of light that is associated with some
// sort of geometry. It's the only type of light that can be "intersected"
// in a scene.
pub trait AreaLight: Light + Geometry {
    // int: the point of interaction
    // w: the direction from which the light is coming (pointed away from the surface)
    fn eval(&self, int: GeomInteraction, w: Vec3<f64>) -> Spectrum;
}
