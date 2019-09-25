use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::vector::Vec3;
use crate::math::ray::Ray;
use crate::shading::material::{Bsdf, Material};
use crate::transform::Transform;
use crate::bvh::{BVH, BVHObject};

use bumpalo::Bump;
use simple_error::{bail, SimpleResult};

use std::collections::HashMap;
use std::mem::{transmute, ManuallyDrop};

// This is used to setup all of the scene information:
pub struct SceneBuilder<'a> {
    pub(super) allocator: Bump,
    // Both of these are manually dropped to safe on memory as they won't
    // be needed when we are actually rendering:
    pub(super) geometry_ids: ManuallyDrop<HashMap<String, &'a dyn Geometry>>,
    pub(super) material_ids: ManuallyDrop<HashMap<String, &'a dyn Material>>,
    pub(super) models: Vec<Model<'a>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn new() -> Self {
        SceneBuilder {
            allocator: Bump::new(),
            geometry_ids: ManuallyDrop::new(HashMap::new()),
            material_ids: ManuallyDrop::new(HashMap::new()),
            models: Vec::new(),
        }
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_geometry<T: Geometry>(&mut self, geometry: T, id: String) -> SimpleResult<()> {
        // First check if we already have an ID that matches this one:
        if self.geometry_ids.contains_key(&id) {
            bail!("Geometry id: \"{}\" is not unique.", id);
        }

        // Allocate the memory. This is dirty and I feel dirty for doing it this way, but I can't figure out
        // a clean way of doing this without the lifetime getting in the way. Rust doesn't support self-referential
        // stuff as the lifetimes get in the way.
        let geometry =
            unsafe { transmute::<&mut T, &'a dyn Geometry>(self.allocator.alloc(geometry)) };
        self.geometry_ids.insert(id, geometry);
        Ok(())
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_material<T: Material>(&mut self, material: T, id: String) -> SimpleResult<()> {
        // First check if we already have an ID that matches this one:
        if self.material_ids.contains_key(&id) {
            bail!("Material id: \"{}\" is not unique.", id);
        }

        let material =
            unsafe { transmute::<&mut T, &'a dyn Material>(self.allocator.alloc(material)) };
        self.material_ids.insert(id, material);
        Ok(())
    }

    pub fn add_model<T: Transform>(
        &mut self,
        geometry_id: &str,
        material_id: &str,
        transform: T,
    ) -> SimpleResult<()> {
        let &geometry = match self.geometry_ids.get(geometry_id) {
            Some(g) => g,
            _ => bail!("Geometry with id: {} was not created before.", geometry_id),
        };
        let &material = match self.material_ids.get(material_id) {
            Some(m) => m,
            _ => bail!("Material with id: {} was not created before.", material_id),
        };

        // Because the transform could be either animated or not, we need to use dynamic dispatch to
        // determine what to do:
        let geom_to_world =
            unsafe { transmute::<&mut T, &'a dyn Transform>(self.allocator.alloc(transform)) };
        self.models.push(Model {
            geometry,
            material,
            geom_to_world,
        });
        Ok(())
    }
}

// A model has information regarding the transformation of geometry in the world.
// This is to allow for basic instancing.
struct Model<'a> {
    pub geometry: &'a dyn Geometry,
    pub material: &'a dyn Material,
    pub geom_to_world: &'a dyn Transform,
}

impl<'a> Model<'a> {
    // The intersection needs to take into account motion:
    pub fn intersect(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> Option<Interaction> {
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        let ray = int_geom_to_world.inverse().ray(ray);
        match self.geometry.intersect(ray, max_time) {
            // Don't forget to transform it back to the original space we care about:
            Some(i) => Some(int_geom_to_world.interaction(i)),
            _ => None,
        }
    }

    // The intersection needs to take into account motion:
    pub fn intersect_test(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> bool {
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        // Then we transform the ray itself and calculate the acceleration values:
        let ray = int_geom_to_world.inverse().ray(ray);
        self.geometry.intersect_test(ray, max_time)
    }

    pub fn get_geom_bound(&self) -> BBox3<f64> {
        self.geometry.get_bound()
    }

    pub fn get_geom_centroid(&self) -> Vec3<f64> {
        self.geometry.get_centroid()
    }

    // TODO: figure out how to handle the centroid for potentially
    // moving objects.
    pub fn get_world_centroid(&self) -> Vec3<f64> {
        self.get_geom_centroid()
    }

    pub fn get_world_bound(&self) -> BBox3<f64> {
        self.geom_to_world.bound_motion(self.geometry.get_bound())
    }
}

impl<'a> BVHObject for Model<'a> {
    type IntParam = ();
    type DataParam = ();

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64, curr_time: f64, _: &Self::IntParam) -> bool {
        Model::intersect_test(self, ray, max_time, curr_time)
    }

    fn intersect(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        curr_time: f64,
        _: &Self::IntParam,
    ) -> Option<Interaction> {
        Model::intersect(self, ray, max_time, curr_time)
    }

    fn get_centroid(&self, _: &Self::DataParam) -> Vec3<f64> {
        self.get_world_centroid()
    }

    fn get_bound(&self, _: &Self::DataParam) -> BBox3<f64> {
        self.get_world_bound()
    }
}

pub struct Scene<'a> {
    allocator: Bump,
    bvh: BVH<Model<'a>>,
}

impl<'a> Scene<'a> {
    // TODO: make this a user param (or not):
    const MAX_MODEL_PER_NODE: usize = 16;

    pub fn new(scene_builder: SceneBuilder<'a>) -> Self {
        Scene {
            allocator: scene_builder.allocator,
            bvh: BVH::new(scene_builder.models, Self::MAX_MODEL_PER_NODE, &()),
        }
    }
}
