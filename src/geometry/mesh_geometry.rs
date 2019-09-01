use crate::geometry::mesh_bvh::MeshBVH;
use crate::geometry::mesh::calc_rayintinfo;
use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::transform::Transform;

use std::cell::Cell;

pub struct MeshGeometry<T: Transform> {
    mesh_bvh: MeshBVH,

    geom_to_world: T,

    // Because surface area calculations are costly,
    // we calculate them only once and cache the result here:
    surface_area: Cell<Option<f64>>,
}

impl<T: Transform> MeshGeometry<T> {
    pub fn new(mesh_bvh: MeshBVH, geom_to_world: T) -> Self {
        MeshGeometry {
            mesh_bvh,
            geom_to_world,
            surface_area: Cell::new(None),
        }
    }
}

impl<T: Transform> Geometry for MeshGeometry<T> {
    fn world_bound(&self, t: f64) -> BBox3<f64> {
        self.geom_to_world.bound_motion(self.mesh_bvh.object_bound(), t)
    }

    fn surface_area(&self) -> f64 {
        // Check if we already calculated this:
        if let Some(s) = self.surface_area.get() {
            return s;
        }

        // Otherwise, we go ahead and calculate the surface area
        // and cache it:
        let s = self.mesh_bvh.mesh().get_surface_area();
        self.surface_area.set(Some(s));
        s
    }

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> bool {
        // First we transform the ray into the appropraite space:
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        // Then we transform the ray itself and calculate the acceleration values:
        let ray = int_geom_to_world.inverse().ray(ray);
        let int_info = calc_rayintinfo(ray);
        self.mesh_bvh.intersect_test(ray, max_time, int_info)
    }

    fn intersect(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> Option<Interaction> {
        // First we transform the ray into the appropraite space:
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        // Then we transform the ray itself and calculate the acceleration values:
        let ray = int_geom_to_world.inverse().ray(ray);
        let int_info = calc_rayintinfo(ray);
        match self.mesh_bvh.intersect(ray, max_time, int_info) {
            // Don't forget to transform it back to the original space we care about:
            Some(i) => Some(int_geom_to_world.interaction(i)),
            _ => None,
        }
    }
}
