pub mod loader;
pub mod mesh;
pub mod mesh_bvh;

use crate::geometry::mesh::Intersection;
use crate::math::bbox::BBox3f;
use crate::math::ray::Ray;

pub trait Geometry {
    fn object_bound(&self) -> BBox3f;
    fn world_bound(&self) -> BBox3f;

    fn intersect_test(&self, ray: Ray, max_time: f32) -> bool;
    fn intersect(&self, ray: Ray, max_time: f32) -> Option<Intersection>;
}
