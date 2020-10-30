pub mod mesh;

use pmath;
use pmath::bbox::BBox3;
use pmath::ray::Ray;
use pmath::vector::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct GeomSurface {
    pub p: Vec3<f64>,  // intersection point
    pub n: Vec3<f64>,  // geometric normal (of triangle)
    pub wo: Vec3<f64>, // direction of intersection leaving the point

    pub t: f64, // the t value of the intersection of the ray

    pub uv: Vec2<f64>,   // uv coordinate at the intersection
    pub dpdu: Vec3<f64>, // vectors parallel to the triangle
    pub dpdv: Vec3<f64>,

    pub shading_n: Vec3<f64>,    // the shading normal at this point
    pub shading_dpdu: Vec3<f64>, // the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3<f64>,
    pub shading_dndu: Vec3<f64>, // the shading dndu, dndv at this point
    pub shading_dndv: Vec3<f64>,
}

/// A geometry is something that can be intersected in the scene.
pub trait Geometry: Sync + 'static {
    /// Perform the different intersections and whatnot:
    fn intersect(&self, ray: Ray<f64>) -> Option<GeomSurface>;
    fn intersect_test(&self, ray: Ray<f64>) -> bool;

    /// Returns the surface area. If `calc_surface_area` wasn't called yet, or if a transform was applied that would
    /// change this, return -1.0.
    fn get_surface_area(&self) -> f64;

    /// Calculates the surface area:
    fn calc_surface_area(&mut self) -> f64;

    /// Returns a bounding box of the geometry:
    fn get_bbox(&self) -> BBox3<f64>;
}
