use crate::bvh::{BVHObject, BVH};
use crate::geometry::Geometry;
use crate::interaction::{GeomIntr, Interaction};
use crate::light::Light;
use crate::shading::material::Material;
use crate::transform::Transf;
use pmath::bbox::BBox3;
use pmath::ray::Ray;
use std::sync::{Arc, Mutex};

//
// ScenePrim
//

/// A public trait that represents a scene primitive
trait ScenePrim {
    fn get_transf(&self) -> Transf;
    fn get_light(&self) -> Option<Arc<dyn Light>>;

    /// Returns the number of primitives contained in this primitive other then itself
    fn num_prims(&self) -> usize;
    fn get_prim_at(&self, i: usize) -> &dyn ScenePrim;

    fn get_bbox(&self) -> BBox3<f64>;
    fn intersect(&self, ray: Ray<f64>) -> Option<Interaction>;
    fn intersect_test(&self, ray: Ray<f64>) -> bool;
}

//
// Lights
//

/// Represents a light in the scene.
pub struct SceneLight {
    light: Arc<dyn Light>,
    transf: Transf,
}

impl SceneLight {
    /// Returns the light as a reference and the transformation of the light (in scene space).
    pub fn get_light_transf(&self) -> (&dyn Light, Transf) {
        (self.light.as_ref(), self.transf)
    }
}

/// Stores all of the lights in a scene.
pub struct SceneLightRegistrar {
    light_list: Mutex<Vec<SceneLight>>,
}

impl SceneLightRegistrar {
    /// Constructs a new SceneLightRegistrar.
    fn new() -> Self {
        SceneLightRegistrar {
            light_list: Mutex::new(Vec::new()),
        }
    }

    /// Adds a light to the registrar, returning the index of said light.
    fn register_light(&self, light: &Arc<dyn Light>, transf: Transf) -> usize {
        let light_list = self.light_list.lock().unwrap();
        let index = light_list.len();
        light_list.push(SceneLight {
            light: light.clone(),
            transf,
        });
        index
    }

    /// Updates the transform of the light in the registrar by multiplying on the left side.
    fn update_ligth(&self, index: usize, transf: Transf) {
        // Acquire a lock for the vector:
        let light_list = self.light_list.lock().unwrap();
        let scene_light = light_list.get_mut(index).unwrap();
        scene_light.transf = transf * scene_light.transf;
    }
}

static SCENE_LIGHT_REGISTRAR: SceneLightRegistrar = SceneLightRegistrar::new();

// Lights are a little different from everything. I need to be able to access all light sources in a scene quickly
// in a single list. This is to make light sampling easier. So, how do we do that? Well, we need to keep some sort of
// "light stack" of transformations. As this could happen to multiple objects outside of regular lights, I'll attach
// support for this to the ScenePrimitive itself as well.

//
// SceneGeom
//

/// A `SceneGeometry` can either be a light source (e.g. a mesh light) or an object with a material.

enum SceneGeomType {
    Material(Arc<dyn Material>),
    Light(Arc<dyn Light>),
}

/// A geometry in the scene. This means a bunch of stuff.
pub struct SceneGeom {
    geom: Arc<dyn Geometry>,
    scene_geom_type: SceneGeomType,
    transf: Transf, // geom to world
}

impl SceneGeom {
    /// Constructs a new `SceneGeom` that has a material associated with it.
    pub fn new_material(
        geom: Arc<dyn Geometry>,
        material: Arc<dyn Material>,
        transf: Transf,
    ) -> Self {
        SceneGeom {
            geom,
            scene_geom_type: SceneGeomType::Material(material),
            transf,
        }
    }

    /// Constructs a new `SceneGeometry` that has a light associated with it.
    pub fn new_light(geom: Arc<dyn Geometry>, light: Arc<dyn Light>, transf: Transf) -> Self {
        SceneGeom {
            geom,
            scene_geom_type: SceneGeomType::Light(light),
            transf,
        }
    }
}

impl ScenePrim for SceneGeom {
    fn get_transf(&self) -> Transf {
        self.transf
    }

    fn get_light(&self) -> Option<Arc<dyn Light>> {
        match self.scene_geom_type {
            SceneGeomType::Light(light) => Some(light),
            _ => None,
        }
    }

    fn num_prims(&self) -> usize {
        0
    }

    fn get_prim_at(&self, _: usize) -> &dyn ScenePrim {
        panic!("SceneGeom doesn't itself contain other ScenePrim objects");
    }

    fn get_bbox(&self) -> BBox3<f64> {
        self.transf.bbox(self.geom.get_bbox())
    }

    fn intersect(&self, ray: Ray<f64>) -> Option<Interaction> {
        let geom_space_ray = self.transf.inverse().ray(ray);
        self.geom
            .intersect(geom_space_ray)
            .map(|o| self.transf.interaction(o))
    }

    fn intersect_test(&self, ray: Ray<f64>) -> bool {
        let geom_space_ray = self.transf.inverse().ray(ray);
        self.geom.intersect_test(geom_space_ray)
    }
}

//
// SceneBVH
//

pub struct SceneBVH {
    bvh: BVH<Arc<dyn ScenePrim>>,
    transf: Transf,
}

impl ScenePrim for SceneBVH {
    fn get_transf(&self) -> Transf {
        self.transf
    }

    fn get_light(&self) -> Option<Arc<dyn Light>> {
        None
    }

    fn num_prims(&self) -> usize {
        self.bvh.get_objects().len()
    }

    fn get_prim_at(&self, i: usize) -> &dyn ScenePrim {
        &self.bvh.get_objects()[i]
    }

    fn get_bbox(&self) -> BBox3<f64> {
        self.transf.bbox(self.bvh.get_bbox())
    }

    fn intersect(&self, ray: Ray<f64>) -> Option<Interaction> {
        let geom_space_ray = self.transf.inverse().ray(ray);
        self.bvh
            .intersect(geom_space_ray, &())
            .map(|o| self.transf.interaction(o))
    }

    fn intersect_test(&self, ray: Ray<f64>) -> bool {
        let geom_space_ray = self.transf.inverse().ray(ray);
        self.bvh.intersect_test(geom_space_ray, &())
    }
}

//
// The "SceneBVHObject" is just an Arc<dyn ScenePrim>:

impl BVHObject for Arc<dyn ScenePrim> {
    type UserData = ();

    fn get_bbox(&self, _: &Self::UserData) -> BBox3<f64> {
        self.as_ref().get_bbox()
    }

    fn intersect_test(&self, ray: Ray<f64>, _: &Self::UserData) -> bool {
        self.as_ref().intersect_test(ray)
    }

    fn intersect(&self, ray: Ray<f64>, _: &Self::UserData) -> Option<Interaction> {
        self.as_ref().intersect(ray)
    }
}

impl ScenePrim for Arc<dyn ScenePrim> {
    fn get_transf(&self) -> Transf {
        self.get_transf()
    }

    fn get_light(&self) -> Option<Arc<dyn Light>> {
        self.get_light()
    }

    fn num_prims(&self) -> usize {
        self.num_prims()
    }

    fn get_prim_at(&self, i: usize) -> &dyn ScenePrim {
        self.get_prim_at(i)
    }

    fn get_bbox(&self) -> BBox3<f64> {
        // Remove the ambiguity:
        self.as_ref().get_bbox()
    }

    fn intersect_test(&self, ray: Ray<f64>) -> bool {
        self.as_ref().intersect_test(ray)
    }

    fn intersect(&self, ray: Ray<f64>) -> Option<Interaction> {
        self.as_ref().intersect(ray)
    }
}
