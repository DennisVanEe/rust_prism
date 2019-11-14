pub mod mesh;
pub mod sphere;

use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::{Vec2, Vec3};
use crate::light::area::AreaLight;
use crate::spectrum::Spectrum;

// Stores the result of an intersection:
#[derive(Clone, Copy)]
pub struct Interaction<'a> {
    pub p: Vec3<f64>,  // intersection point
    pub n: Vec3<f64>,  // geometric normal (of triangle)
    pub wo: Vec3<f64>, // direction of intersection leaving the point

    pub t: f64, // the t value of the intersection of the ray (not time).

    pub uv: Vec2<f64>,   // uv coordinate at the intersection
    pub dpdu: Vec3<f64>, // vectors parallel to the triangle
    pub dpdv: Vec3<f64>,

    pub shading_n: Vec3<f64>,    // the shading normal at this point
    pub shading_dpdu: Vec3<f64>, // the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3<f64>,
    pub shading_dndu: Vec3<f64>, // the shading dndu, dndv at this point
    pub shading_dndv: Vec3<f64>,

    pub light: Option<&'a dyn AreaLight>, // intersection might be a light source if it's an area light
}

impl<'a> Interaction<'a> {
    // Calculates the emitted radiance from the surface in the given direction.
    // If there is no light attached to it, then it returns black
    pub fn emit_radiance(self, w: Vec3<f64>) -> Spectrum {
        match self.light {
            Some(light) => light.eval(self, w),
            None => Spectrum::black(),
        }
    }
}

// The basic geometry trait defines the geometry that PRISM can intersect.

pub trait Geometry {
    // The bounds in geometry space:
    fn get_bound(&self) -> BBox3<f64>;
    // The cetroid in geometry space:
    fn get_centroid(&self) -> Vec3<f64>;
    // This just calculates the surface area of whatever geometry we care about:
    fn get_surface_area(&self) -> f64;

    // Need to specify max_time so we can potentially return early if
    // the intersection is beyond. Curr_time is also needed if the
    // object moves.

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64) -> bool;
    fn intersect(&self, ray: Ray<f64>, max_time: f64) -> Option<Interaction>;
}
