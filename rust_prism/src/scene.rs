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

// TODO: handle the case where we may clone values with the LightHandle. The problem is simple, if we clone a handle that
// contains a light handle, then we should register it (heck, maybe just do this everytime we clone? but then we need a 
// unique light for every single object, or at least, a unique SceneLight in the SceneLightRegistrar. Hmmm...)

/// A single scene light holds the light itself as well as any transformations that belong to the light. Because we
/// build on the lights, the transf that belongs to each light will be a "world" position of said light. This is
/// public to allow light pickers to pick lights based on their property and whatnot.
pub struct SceneLight {
    light: Arc<dyn Light>,
    transf: Transf,
}

impl SceneLight {
    /// Returns the light as a reference and the transformation of the light (in world space).
    pub fn get_light_transf(&self) -> (&dyn Light, Transf) {
        (self.light.as_ref(), self.transf)
    }
}

/// There is only 1 light pool allowed per scene, and every light that can be created has to go through the LightPool
/// (i.e, it needs to registger with it).
pub struct SceneLightRegistrar {
    light_list: Mutex<Vec<SceneLight>>,
}

impl SceneLightRegistrar {
    fn new() -> Self {
        SceneLightRegistrar {
            light_list: Mutex::new(Vec::new()),
        }
    }

    /// Adds a light to the registrar, returning the index of said light.
    fn add_light(&self, light: &Arc<dyn Light>, transf: Transf) -> usize {
        // Acquire a lock for the vector:
        let light_list = self.light_list.lock().unwrap();

        let index = light_list.len();
        light_list.push(SceneLight {
            light: light.clone(),
            transf,
        });
        index
    }

    /// Updates the transform of the light in the registrar by multiplying on the left side.
    fn update_ligth_transf(&self, index: usize, transf: Transf) {
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

#[derive(Clone)]
pub struct LightHandle {
    light: Arc<dyn Light>, // the light itself
    index: Option<usize>,          // index into the scene light registrar
}

impl LightHandle {
    /// Creates a new `LightHandle`. This function should be called by the light creation function (e.g. a function
    /// for creating point lights, etc.) and never by the end user through a script or something.
    pub fn new(light: Arc<dyn Light>) -> Self {
        LightHandle { light, index: None }
    }

    /// Once the light becomes a part of a `ScenePrimitive` object, this should be called to register it. It should
    /// get registered with an original transformation.
    fn register_light(&mut self, transf: Transf) {
        self.index = Some(SCENE_LIGHT_REGISTRAR.add_light(&self.light, transf));
    }

    /// If the ScenePrimitive joins something that changes it's position in the global space, make sure to update
    /// the light as appropriate:
    fn update_transf(&mut self, transf: Transf) {
        // For now, we just ignore the case where index wasn't set:
        if let Some(index) = self.index {
            SCENE_LIGHT_REGISTRAR.update_ligth_transf(index, transf);
        }
    }
}

/// A scene primitive is anything that can
trait ScenePrimitive: 'static {
    // All of the stuff that is required for intersections:

    fn get_bbox(&self) -> BBox3<f64>;
    fn intersect_test(&self, ray: Ray<f64>) -> bool;
    fn intersect(&self, ray: Ray<f64>) -> Option<GeomSurface>;

    /// If anything part of the scene primitive needs to update it's position in the world.
    fn update_transf(&mut self, transf: Transf);
}

type ScenePrimitiveHandle = Arc<dyn ScenePrimitive>;
type GeometryHandle = Arc<dyn Geometry>;
type MaterialHandle = Arc<dyn Material>;

//
// SceneGeom
//

/// A `SceneGeometry` can either be a light source (e.g. a mesh light) or an object with a material.

enum SceneGeometryType {
    Material(MaterialHandle),
    Light(LightHandle),
}

/// A geometry in the scene. This means a bunch of stuff.
pub struct SceneGeometry {
    geometry: GeometryHandle,
    scene_geometry_type: SceneGeometryType,
}

impl SceneGeometry {
    /// Constructs a new `SceneGeometry` that has a material associated with it.
    pub fn new_material(
        geometry: &GeometryHandle,
        material: &MaterialHandle,
    ) -> ScenePrimitiveHandle {
        Arc::new(SceneGeometry {
            geometry: geometry.clone(),
            scene_geometry_type: SceneGeometryType::Material(material.clone()),
        })
    }

    /// Constructs a new `SceneGeometry` that has a light associated with it.
    pub fn new_light(geometry: &GeometryHandle, light: &LightHandle) -> ScenePrimitiveHandle {
        // WE don't need to register it yet. Once it is attached to a BVH, then we do it.
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

    fn update_transf(&mut self, transf: Transf) {
        // We only perform the update if we have a light:
        if let SceneGeometryType::Light(light_handle) = &mut self.scene_geometry_type {
            match light_handle.index {
                Some(index) => light_handle.update_transf(transf),
                None => light_handle.register_light(transf),
            }
        }
    }
}

//
// SceneBVHObject
//

/// A `SceneBVHObject` is an object that can be inserted into a bvh. It is made up of a `ScenePrimitiveHandle` and a
/// `Transf` to transform it (relative to the BVH itself, of course).
#[derive(Clone)]
struct SceneBVHObject {
    object: ScenePrimitiveHandle,
    transf: Transf,
}

impl SceneBVHObject {
    /// Constructs a new `SceneBVHObject`. Note that the `ScenePrimitiveHandle` isn't moved, it's cloned so that we
    /// can mimic some form of instancing.
    pub fn new(object: &ScenePrimitiveHandle, transf: Transf) -> Self {
        let object = object.clone();
        object.update_transf(transf);
        SceneBVHObject {
            object,
            transf,
        }
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
// SceneBVHBuilder
//

/// A `SceneBVHBuilder` is an object used to collect different `SceneBVHObject`s and spit out a `SceneBVH` as a final
/// result.
pub struct SceneBVHBuilder {
    objects: Vec<SceneBVHObject>,
}

impl SceneBVHBuilder {
    /// Constructs a new `SceneBVHBuilder`
    pub fn new() -> Self {
        SceneBVHBuilder {
            objects: Vec::new(),
        }
    }

    /// Adds an object to the bvh builder.
    pub fn add_object(&mut self, obj: SceneBVHObject) {
        self.objects.push(obj);
    }

    /// Creates the `SceneBVH`. Note that this will clear the objects directory, so you can't call it multiple times.
    /// It'll also panic if no items were added. Returns
    pub fn create_bvh(&mut self, max_objects_per_leaf: usize) -> ScenePrimitiveHandle {
        if self.objects.is_empty() {
            panic!("Error creating BVH, SceneBVHBuilder has no objects.");
        }

        let bvh = BVH::new(&self.objects, max_objects_per_leaf, &());
        self.objects.clear();
        Arc::new(bvh)
    }
}
