use crate::bvh::{BVHObject, BVH};
use crate::geometry::mesh::{Mesh, Triangle};
use crate::geometry::{self, Interaction, RayIntInfo};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::Vec3;

use std::mem;
use std::mem::MaybeUninit;

impl BVHObject for Triangle {
    type Param = Mesh;

    fn move_out(&mut self) -> Self {
        *self
    }

    fn intersect_test(&self, ray: Ray<f64>, int_info: RayIntInfo, param: &Mesh) -> bool {
        self.intersect_test(ray, int_info, param)
    }

    fn intersect(&self, ray: Ray<f64>, int_info: RayIntInfo, param: &Mesh) -> Option<Interaction> {
        self.intersect(ray, int_info, param)
    }

    fn transf_bvh_to_object(
        &self,
        ray: Ray<f64>,
        int_info: RayIntInfo,
        _: &Mesh,
    ) -> (Ray<f64>, RayIntInfo) {
        // Essentially a no-op
        (ray, int_info)
    }

    fn get_centroid(&self, param: &Mesh) -> Vec3<f64> {
        self.calc_centroid(param)
    }

    fn get_bound(&self, param: &Mesh) -> BBox3<f64> {
        self.calc_bound(param)
    }
}

pub struct BVHMesh {
    mesh: Mesh,
    bvh: BVH<Triangle>,
}

impl BVHObject for BVHMesh {
    type Param = (); // DOn't need anything extra

    fn move_out(&mut self) -> Self {
        let mut result = unsafe { MaybeUninit::uninit().assume_init() };
        mem::swap(self, &mut result);
        result
    }

    fn intersect_test(&self, ray: Ray<f64>, int_info: RayIntInfo, _: &()) -> bool {
        self.bvh.intersect_test(ray, int_info, &self.mesh)
    }

    fn intersect(&self, ray: Ray<f64>, int_info: RayIntInfo, _: &()) -> Option<Interaction> {
        self.bvh.intersect(ray, int_info, &self.mesh)
    }

    fn transf_bvh_to_object(
        &self,
        ray: Ray<f64>,
        int_info: RayIntInfo,
        _: &(),
    ) -> (Ray<f64>, RayIntInfo) {
        let mesh_to_world = self.mesh.get_transform();
        // If it's not animated, it should already be cached:
        if !mesh_to_world.is_animated() {
            return (ray, int_info);
        }

        // Otherwise we have to perform the animated transform:
        let int_mesh_to_world = mesh_to_world.interpolate(ray.time);
        let ray = int_mesh_to_world.inverse().ray(ray);
        // Recalculate the int_info:
        let int_info = geometry::calc_rayintinfo(ray);
        (ray, int_info)
    }

    fn get_centroid(&self, _: &()) -> Vec3<f64> {
        // Sum up all of the areas and all the centers:
        let (area, centroid) =
            self.mesh
                .triangles
                .iter()
                .fold((0.0, Vec3::zero()), |(a, c), triangle| {
                    let centroid = triangle.calc_centroid(&self.mesh);
                    let area = triangle.calc_area(&self.mesh);
                    (a + area, c + centroid.scale(area))
                });
        centroid.scale(1.0 / area)
    }

    fn get_bound(&self, _: &()) -> BBox3<f64> {
        self.bvh.get_bound()
    }
}
