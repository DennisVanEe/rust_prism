pub mod mesh;

use crate::transform::Transf;
use embree;
use math;
use math::ray::Ray;
use math::vector::{Vec2, Vec3};
use simple_error::SimpleResult;

#[derive(Clone, Copy, Debug)]
pub struct GeomInteraction {
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

    pub material_id: u32, // An index to the material specified with this interaction
}

/// A geometry is something that can be intersected in the scene.
pub trait Geometry: 'static {
    fn transform(&mut self, transf: Transf);

    fn create_embree_geometry(&mut self, device: embree::Device) -> SimpleResult<embree::Geometry>;
    fn delete_embree_geometry(&mut self, device: embree::Device) -> SimpleResult<()>;
    fn get_embree_geometry(&self) -> embree::Geometry;

    fn get_surface_area(&self) -> f64;
    fn calc_surface_area(&mut self) -> f64;

    /// Calculates a geometric interaction given the ray that led to the interaction and the hit.
    fn calc_interaction(
        &self,
        ray: Ray<f64>,
        hit: embree::Hit<f64>,
        material_id: u32,
    ) -> GeomInteraction;
}
