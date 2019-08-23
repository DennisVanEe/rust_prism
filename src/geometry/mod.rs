pub mod loader;
pub mod mesh;
pub mod mesh_bvh;
pub mod sphere;

use crate::geometry::mesh::Intersection;
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;

// The basic geometry trait defines the geometry that PRISM can intersect.

pub trait Geometry {
    fn object_bound(&self) -> BBox3<f64>;
    fn world_bound(&self) -> BBox3<f64>;

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64) -> bool;
    fn intersect(&self, ray: Ray<f64>, max_time: f64) -> Option<Intersection>;
}