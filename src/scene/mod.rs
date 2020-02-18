pub mod scene_builder;
use scene_builder::SceneBuilder;

use crate::bvh::{BVHObject, BVH};
use crate::geometry::{GeomInteraction, Geometry};
use crate::light::area::AreaLight;
use crate::light::Light;
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::{Vec2, Vec3};
use crate::shading::material::Material;
use crate::spectrum::Spectrum;
use crate::transform::Transform;

use bumpalo::Bump;

// Specifies the type that the object in the scene exhibits.
pub enum SceneObjectType<'a> {
    Light(&'a dyn AreaLight),   // The object should be treated as an area light.
    Material(&'a dyn Material), // The object has a material attached to it.
                                // A material can emit radiance, but it won't be treated like a light
                                // so it won't be importance sampled.
}

// Information (in scene space) of a ray intersecting the object.
pub struct SceneInteraction<'a> {
    pub geom: GeomInteraction, // The geometry interaction of the object.
    pub obj_type: SceneObjectType<'a>, // The object type (see above) of the object intersected.
}

impl<'a> SceneInteraction<'a> {
    // Calculates the radiance at the interaction point in the direction
    // given by w. If the object is not a ObjectType::Light, then it always
    // returns black
    pub fn light_radiance(self, w: Vec3<f64>) -> Spectrum {
        match self.obj_type {
            SceneObjectType::Light(l) => l.eval(self, w),
            _ => Spectrum::black(),
        }
    }
}

// A Scene Object is a special type of object that exists in the scene.
struct SceneObject<'a> {
    geom: &'a dyn Geometry, // The geometry that represents the scene object.
    obj_type: SceneObjectType<'a>, // The type of information that is associated with the object.
    geom_to_scene: &'a dyn Transform, // Transforms from geometry to scene space.
}

impl<'a> BVHObject for SceneObject<'a> {
    type IntParam = ();
    type DataParam = ();
    type IntResult = SceneInteraction<'a>;

    fn intersect_test(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        curr_time: f64,
        _: &Self::IntParam,
    ) -> bool {
        let int_geom_to_scene = self.geom_to_scene.interpolate(curr_time);
        let ray = int_geom_to_scene.inverse().ray(ray);
        self.geom.intersect_test(ray, max_time)
    }

    fn intersect(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        curr_time: f64,
        _: &Self::IntParam,
    ) -> Option<SceneInteraction> {
        let int_geom_to_scene = self.geom_to_scene.interpolate(curr_time);
        let ray = int_geom_to_scene.inverse().ray(ray);
        // Then we intersect the object and check if we hit something:
        match self.geom.intersect(ray, max_time) {
            Some(i) => Some(SceneInteraction {
                geom: i,
                obj_type: self.obj_type,
            }),
            _ => None,
        }
    }

    fn get_centroid(&self, _: &Self::DataParam) -> Vec3<f64> {
        self.geom.get_centroid()
    }

    fn get_bound(&self, _: &Self::DataParam) -> BBox3<f64> {
        self.geom_to_scene.bound_motion(self.geom.get_bound())
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
    allocator: Bump, // Holds any of the memory that we may have needed when rendering.
    lights: Vec<SceneLight<'a>>, // Holds the importance sampled light sources in the scene.
    bvh: BVH<SceneObject<'a>>, // Holds everything that can be intersected in the scene.
}

impl<'a> Scene<'a> {
    const MAX_MODEL_PER_NODE: usize = 16;

    pub fn new(scene_builder: SceneBuilder<'a>) -> Self {
        Scene {
            allocator: scene_builder.allocator,
            lights: Vec::new(), // TODO: add support for lights here
            bvh: BVH::new(scene_builder.models, Self::MAX_MODEL_PER_NODE, &()),
        }
    }

    // These intersection tests are in world space:

    // The intersect function also returns a reference to the material that belongs
    // to the object that was intersected. This way, the integrator can decide whether
    // or not to construct a Bsdf object.
    pub fn intersect(&self, ray: Ray<f64>, max_t: f64, curr_time: f64) -> Option<SceneInteraction> {
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
