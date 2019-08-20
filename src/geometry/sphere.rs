
use crate::geometry::Geometry;
use crate::math::transform::{AnimatedTransform, Transform};
use crate::math::bbox::BBox3f;

pub struct Sphere {
    // These are pretty easy to invert, so we don't 
    // have to bother storing both bits of information in this case:
    geom_to_world: Transform<f32>,

    radius: f32,
    z_min: f32,
    z_max: f32,
    theta_min: f32,
    theta_max: f32,
    phi_max: f32,
}

pub struct AnimatedSphere {
    // Stores the geometry to world and world to geometry transformations.
    // We need both because it's costly to invert an animated transform (I think)...
    geom_to_world: AnimatedTransform<f32>,
    world_to_geom: AnimatedTransform<f32>,

    radius: f32,
    z_min: f32,
    z_max: f32,
    theta_min: f32,
    theta_max: f32,
    phi_max: f32,
}

impl AnimatedSphere {
    pub fn new()
}