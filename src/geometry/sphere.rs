
use crate::geometry::Geometry;
use crate::math::transform::{AnimatedTransform, Transform};
use crate::math::bbox::BBox3;

pub struct Sphere {
    // These are pretty easy to invert, so we don't 
    // have to bother storing both bits of information in this case:
    geom_to_world: Transform<f64>,

    radius: f64,
    z_min: f64,
    z_max: f64,
    theta_min: f64,
    theta_max: f64,
    phi_max: f64,
}

pub struct AnimatedSphere {
    // Stores the geometry to world and world to geometry transformations.
    // We need both because it's costly to invert an animated transform (I think)...
    geom_to_world: AnimatedTransform<f64>,
    world_to_geom: AnimatedTransform<f64>,

    radius: f64,
    z_min: f64,
    z_max: f64,
    theta_min: f64,
    theta_max: f64,
    phi_max: f64,
}

impl AnimatedSphere {
    pub fn new()
}