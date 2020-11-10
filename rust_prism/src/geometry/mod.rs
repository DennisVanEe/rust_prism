pub mod mesh;

use crate::interaction::Surface;
use pmath;
use pmath::bbox::BBox3;
use pmath::ray::Ray;

/// A geometry is something that can be intersected in the scene.
pub trait Geometry: Sync + 'static {
    /// Perform the different intersections and whatnot:
    fn intersect(&self, ray: Ray<f64>) -> Option<Surface>;
    fn intersect_test(&self, ray: Ray<f64>) -> bool;

    /// Returns the surface area. If `calc_surface_area` wasn't called yet, or if a transform was applied that would
    /// change this, return -1.0.
    fn get_surface_area(&self) -> f64;

    /// Calculates the surface area:
    fn calc_surface_area(&mut self) -> f64;

    /// Returns a bounding box of the geometry:
    fn get_bbox(&self) -> BBox3<f64>;
}
