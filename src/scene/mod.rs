pub mod scene_builder;

use crate::bvh::{BVHObject, BVH};
use crate::geometry::{Geometry, GeometryInteraction};
use crate::light::area::AreaLight;
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::{Vec2, Vec3};
use crate::shading::material::{Bsdf, Material};
use crate::spectrum::Spectrum;
use crate::transform::Transform;

use bumpalo::Bump;

// The interaction that 
struct SceneModelInteraction<'a> {
    // The geometry interaction portion:
    geometry: GeometryInteraction,
    light: Option<&'a dyn Light>,
}

// Whether or not the scene model 
enum SceneModelType<'a> {
    Light(&'a dyn Light),
    Material(&'a dyn Material),
}

// A model has information regarding the transformation of geometry in the world.
// This is to allow for basic instancing.
struct SceneModel<'a> {
    geometry: &'a dyn Geometry,
    type: SceneModelType<'a>,
    geom_to_world: &'a dyn Transform,
}

impl<'a> BVHObject for SceneModel<'a> {
    type IntParam = ();
    type DataParam = ();
    type IntResult = SceneModelInteraction<'a>,

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
    ) -> Option<SceneModelInteraction<'a>> {
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        let ray = int_geom_to_world.inverse().ray(ray);
        let geom_int = match self.geometry.intersect(ray, max_time) {
            // Don't forget to transform it back to the original space we care about:
            Some(i) => Some(int_geom_to_world.interaction(i)),
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
    bvh: BVH<SceneGeometry<'a>>,
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
    ) -> Option<(Interaction, &dyn Material)> {
        // First we traverse the BVH and get what we want:
        match self.bvh.intersect(ray, max_t, curr_time, &()) {
            Some((i, o)) => Some((i, o.material)),
            _ => None,
        }
    }

    pub fn intersect_test(&self, ray: Ray<f64>, max_t: f64, curr_time: f64) -> bool {
        self.bvh.intersect_test(ray, max_t, curr_time, &())
    }
}
