use crate::embree::{BufferType, Format, GeometryPtr, GeometryType, DEVICE};
use crate::math::numbers::Float;
use crate::math::ray::Ray;
use crate::math::util;
use crate::math::vector::{Vec2, Vec3};
use crate::transform::Transf;
use std::mem;
use std::os::raw;

#[derive(Clone, Copy, Debug)]
pub struct GeometryInteraction {
    pub p: Vec3<f64>,  /// intersection point
    pub n: Vec3<f64>,  /// geometric normal (of triangle)
    pub wo: Vec3<f64>, /// direction of intersection leaving the point

    pub t: f64, /// the t value of the intersection of the ray

    pub uv: Vec2<f64>,   /// uv coordinate at the intersection
    pub dpdu: Vec3<f64>, /// vectors parallel to the triangle
    pub dpdv: Vec3<f64>,

    pub shading_n: Vec3<f64>,    /// the shading normal at this point
    pub shading_dpdu: Vec3<f64>, /// the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3<f64>,
    pub shading_dndu: Vec3<f64>, /// the shading dndu, dndv at this point
    pub shading_dndv: Vec3<f64>,

    pub material_id: u32, /// An index to the material specified with this interaction
}
pub trait Geometry {

    /// Transforms the local geometry given the transform.
    fn transform(&mut self, transf: Transf);

    /// Create and destroy geometry information here.
    fn create_embree_geometry(&mut self) -> GeometryPtr;
    fn delete_embree_geometry(&mut self);

    /// Returns a geometry pointer for the embree scene.
    fn get_embree_geom(&self) -> GeometryPtr;

    // Creates a surface area and whatnot.
    fn get_surface_area(&self) -> f64;
    fn calc_surface_area(&mut self) -> f64;
}
