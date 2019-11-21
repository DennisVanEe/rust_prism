pub mod scene_builder;

use self::scene_builder::SceneBuilder;

use crate::bvh::{BVHObject, BVH};
use crate::geometry::{Geometry, GeometryInteraction};
use crate::light::area::AreaLight;
use crate::light::Light;
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::{Vec2, Vec3};
use crate::shading::material::Material;
use crate::spectrum::Spectrum;
use crate::transform::Transform;

use bumpalo::Bump;

// The type of SceneModel we are dealing with (an area light
// or a material):
enum SceneObjectType<'a> {
    Light(&'a dyn AreaLight),
    Material(&'a dyn Material),
}

// The interaction that
struct SceneObjectInteraction<'a> {
    // The geometry interaction portion:
    geometry: GeometryInteraction,
    obj_type: SceneObjectType<'a>,
}

impl<'a> SceneObjectInteraction<'a> {
    // If the scene object emits any radiance, it returns it, otherwise
    // it returns black:
    pub fn emit_radiance(self, w: Vec3<f64>) -> Spectrum {
        match self.obj_type {
            SceneObjectType::Light(l) => l.eval(self, w),
            _ => Spectrum::black(),
        }
    }
}

// A model has information regarding the transformation of geometry in the world.
// This is to allow for basic instancing.
struct SceneObject<'a> {
    geometry: &'a dyn Geometry, // The geometry that represents the scene object
    obj_type: SceneObjectType<'a>, // The type of information that is associated with the object
    geom_to_world: &'a dyn Transform, // The transform of the scene object
}

impl<'a> BVHObject for SceneObject<'a> {
    type IntParam = ();
    type DataParam = ();
    type IntResult = SceneObjectInteraction<'a>;

    fn intersect_test(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        curr_time: f64,
        _: &Self::IntParam,
    ) -> bool {
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        // Then we transform the ray itself and calculate the acceleration values:
        let ray = int_geom_to_world.inverse().ray(ray);
        self.geometry.intersect_test(ray, max_time)
    }

    fn intersect(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        curr_time: f64,
        _: &Self::IntParam,
    ) -> Option<SceneObjectInteraction> {
        // First we transform the ray to the object's local space:
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        let ray = int_geom_to_world.inverse().ray(ray);
        // Then we intersect the object and check if we hit something:
        match self.geometry.intersect(ray, max_time) {
            Some(i) => Some(SceneObjectInteraction {
                geometry: i,
                obj_type: self.obj_type,
            }),
            _ => None,
        }
    }

    fn get_centroid(&self, _: &Self::DataParam) -> Vec3<f64> {
        self.geometry.get_centroid()
    }

    fn get_bound(&self, _: &Self::DataParam) -> BBox3<f64> {
        self.geom_to_world.bound_motion(self.geometry.get_bound())
    }
}

// A SceneLight is a light with information regarding the transformation of the light in the
// world. This is to allow for basic instancing of lights:
struct SceneLight<'a> {
    light: &'a dyn Light,
    light_to_scene: &'a dyn Transform,
}

impl<'a> SceneLight<'a> {
    // Transforms everything to the light's space:
    fn sample(
        &self,
        surface_point: Vec3<f64>,
        time: f64,
        u: Vec2<f64>,
    ) -> (Spectrum, Vec3<f64>, f64) {
        let int_light_to_scene = self.light_to_scene.interpolate(time);
        let surface_point = int_light_to_scene.inverse().point(surface_point);

        let (radiance, light_point, pdf) = self.light.sample(surface_point, time, u);

        // Make sure to transform the light point to scene space:
        (radiance, int_light_to_scene.point(light_point), pdf)
    }
}

pub struct Scene<'a> {
    allocator: Bump,
    //lights: Vec<SceneLight<'a>>,
    bvh: BVH<SceneObject<'a>>,
}

impl<'a> Scene<'a> {
    const MAX_MODEL_PER_NODE: usize = 16;

    pub fn new(scene_builder: SceneBuilder<'a>) -> Self {
        Scene {
            allocator: scene_builder.allocator,
            bvh: BVH::new(scene_builder.models, Self::MAX_MODEL_PER_NODE, &()),
        }
    }

    // These intersection tests are in world space:

    // The intersect function also returns a reference to the material that belongs
    // to the object that was intersected. This way, the integrator can decide whether
    // or not to construct a Bsdf object.
    pub fn intersect(
        &self,
        ray: Ray<f64>,
        max_t: f64,
        curr_time: f64,
    ) -> Option<SceneObjectInteraction> {
        // First we traverse the BVH and get what we want:
        match self.bvh.intersect(ray, max_t, curr_time, &()) {
            Some((i, _)) => Some(i),
            _ => None,
        }
    }

    pub fn intersect_test(&self, ray: Ray<f64>, max_t: f64, curr_time: f64) -> bool {
        self.bvh.intersect_test(ray, max_t, curr_time, &())
    }
}
