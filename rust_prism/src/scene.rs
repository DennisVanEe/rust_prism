use crate::bvh::{BVHObject, BVH};
use crate::geometry::{GeomSurface, Geometry};
use crate::light::Light;
use crate::shading::material::Material;
use crate::transform::Transf;
use pmath::bbox::BBox3;
use pmath::ray::Ray;
use std::sync::{Arc, Mutex};

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

enum SceneGeometryType {
    Material(Arc<dyn Material>),
    Light(Arc<dyn Light>),
}

/// A geometry in the scene. This means a bunch of stuff.
pub struct SceneGeometry {
    geometry: Arc<dyn Geometry>,
    scene_geometry_type: SceneGeometryType,
}

impl SceneGeometry {
    /// Constructs a new `SceneGeometry` that has a material associated with it.
    pub fn new_material(
        geometry: &Arc<dyn Geometry>,
        material: &Arc<dyn Material>,
    ) -> Arc<dyn ScenePrimitive> {
        Arc::new(SceneGeometry {
            geometry: geometry.clone(),
            scene_geometry_type: SceneGeometryType::Material(material.clone()),
        })
    }

    /// Constructs a new `SceneGeometry` that has a light associated with it.
    pub fn new_light(
        geometry: &Arc<dyn Geometry>,
        light: &Arc<dyn Light>,
    ) -> Arc<dyn ScenePrimitive> {
        Arc::new(SceneGeometry {
            geometry: geometry.clone(),
            scene_geometry_type: SceneGeometryType::Light(light.clone()),
        })
    }
}

impl ScenePrimitive for SceneGeometry {
    fn get_bbox(&self) -> BBox3<f64> {
        self.geometry.get_bbox()
    }

    fn intersect_test(&self, ray: Ray<f64>) -> bool {
        self.geometry.intersect_test(ray)
    }

    fn intersect(&self, ray: Ray<f64>) -> Option<GeomSurface> {
        self.geometry.intersect(ray)
    }
}

//
// SceneBVHObject
//

/// A `SceneBVHObject` is an object that can be inserted into a bvh. It is made up of a `ScenePrimitiveHandle` and a
/// `Transf` to give it a trasnformation. It also has a special handle that allows it to register and update any lights
/// that may belong to it.
#[derive(Clone)]
struct SceneBVHObject {
    object: Arc<dyn ScenePrimitive>,
    transf: Transf,
}

impl SceneBVHObject {
    /// Constructs a new `SceneBVHObject`. Note that the `ScenePrimitiveHandle` isn't moved, it's cloned so that we
    /// can mimic some form of instancing.
    pub fn new(object: &ScenePrimitiveHandle, transf: Transf) -> Self {
        let object = object.clone();
        object.update_transf(transf);
        SceneBVHObject { object, transf }
    }
}

impl BVHObject for SceneBVHObject {
    type UserData = ();

    fn get_bbox(&self, _: &Self::UserData) -> BBox3<f64> {
        self.object.get_bbox()
    }

    fn intersect_test(&self, ray: Ray<f64>, _: &Self::UserData) -> bool {
        self.object.intersect_test(ray)
    }

    fn intersect(&self, ray: Ray<f64>, _: &Self::UserData) -> Option<GeomSurface> {
        self.object.intersect(ray)
    }
}

//
// SceneBVH
//

// Just make sure it can itself be a `ScenePrimitive`.
impl ScenePrimitive for BVH<SceneBVHObject> {
    fn get_bbox(&self) -> BBox3<f64> {
        self.get_bbox()
    }

    fn intersect_test(&self, ray: Ray<f64>) -> bool {
        self.intersect_test(ray, &())
    }

    fn intersect(&self, ray: Ray<f64>) -> Option<GeomSurface> {
        self.intersect(ray, &())
    }
}

//
// ScenePrimitive
//

// pub trait ScenePrimitive {
//     fn intersect(ray: Ray<f64>) -> Option<GeomSurface>;
//     fn
// }

//
// SceneObject
//

pub struct SceneObject {
    primitive: Arc<dyn ScenePrimitive>,
}

//
// InstanceBuilder
//

type Instance = BVH;

/// An object used to construct an instance. An instance is made  up of scene objects.
pub struct InstanceBuilder {
    objects: Vec<SceneBVHObject>,
}

impl InstanceBuilder {
    pub fn new() -> Self {
        InstanceBuilder {
            objects: Vec::new(),
        }
    }

    /// Adds an object to the bvh builder.
    pub fn add_scene_object(&mut self, obj: SceneBVHObject) {
        self.objects.push(obj);
    }

    /// Creates the `SceneBVH`. Note that this will clear the objects directory, so you can't call it multiple times.
    /// It'll also panic if no items were added. Returns
    pub fn create_instance(&mut self, max_objects_per_leaf: usize) -> ScenePrimitiveHandle {
        if self.objects.is_empty() {
            panic!("Error creating BVH, SceneBVHBuilder has no objects.");
        }

        let bvh = BVH::new(&self.objects, max_objects_per_leaf, &());
        self.objects.clear();
        Arc::new(bvh)
    }
}
