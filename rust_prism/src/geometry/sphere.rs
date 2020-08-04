use crate::geometry::{GeomInteraction, Geometry};
use crate::transform::Transf;
use embree;
use pmath::ray::Ray;
use pmath::vector::{Vec2, Vec3, Vec4};
use std::mem;

/// Represents a collection of spheres.
pub struct Spheres {
    spheres: Vec<Vec4<f32>>, // Spheres are represented as (x, y, z, r) groupings.
    transf: Transf,          // In order to support any transformation, we need to store it here.
    embree_geom: embree::Geometry,
}

impl Spheres {
    /// Creates a new collection of spheres. For one sphere, use the `new_one` function. Each Vec4<f32>
    /// represents a sphere as (x, y, z, r).
    pub fn new(spheres: Vec<Vec4<f32>>) -> Self {
        Spheres {
            spheres,
            transf: Transf::new_identity(),
            embree_geom: embree::Geometry::new_null(),
        }
    }

    pub fn new_one(pos: Vec3<f32>, radius: f32) -> Self {
        Spheres {
            spheres: vec![Vec4::from_vec3(pos, radius)],
            transf: Transf::new_identity(),
            embree_geom: embree::Geometry::new_null(),
        }
    }
}

impl Geometry for Spheres {
    /// Permanently applies the transformation to the data of the mesh.
    fn transform(&mut self, transf: Transf) {
        self.transf = transf * self.transf;
    }

    fn create_embree_geometry(&mut self) -> embree::Geometry {
        // Delete the device first.
        self.delete_embree_geometry();

        let embree_geom = embree::new_geometry(embree::GeometryType::SpherePoint);
        embree::set_shared_geometry_buffer(
            embree_geom,
            embree::BufferType::Vertex,
            0,
            embree::Format::Float4,
            self.spheres.as_ptr(),
            0,
            mem::size_of::<Vec4<f32>>(),
            self.spheres.len(),
        );

        self.embree_geom = embree_geom;
        embree_geom
    }

    fn delete_embree_geometry(&mut self) {
        if self.embree_geom.is_null() {
            return;
        }

        let result = embree::release_geometry(self.embree_geom);
        self.embree_geom = embree::Geometry::new_null();
        result
    }

    /// Returns the current RTCGeometry.
    fn get_embree_geometry(&self) -> embree::Geometry {
        self.embree_geom
    }

    fn get_surface_area(&self) -> f64 {
        0.0 // TODO: calculate surface area given transformation
    }

    /// Calculates the surface area of the specific mesh.
    fn calc_surface_area(&mut self) -> f64 {
        0.0 // TODO: calculate surface area given transformation
    }

    fn calc_interaction(
        &self,
        ray: Ray<f64>,
        hit: embree::Hit<f64>,
        material_id: u32,
    ) -> GeomInteraction {
        // TODO: actually calculate this properly.

        // Get the primitive:
        let sphere = self.spheres[hit.prim_id as usize];

        // Calculate the geometric normal:
        let n = hit.ng.normalize();

        let t = ray.t_far;

        let wo = -ray.dir;

        GeomInteraction {
            p: Vec3::zero(),
            n,
            wo,
            t,
            uv: Vec2::zero(),
            dpdu: Vec3::zero(),
            dpdv: Vec3::zero(),
            shading_n: n,
            shading_dpdu: Vec3::zero(),
            shading_dpdv: Vec3::zero(),
            shading_dndu: Vec3::zero(),
            shading_dndv: Vec3::zero(),
            material_id,
        }
    }
}
