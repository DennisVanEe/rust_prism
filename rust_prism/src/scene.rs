use crate::geometry::{GeomInteraction, Geometry};
use crate::light::Light;
use crate::transform::Transf;
use embree;
use pmath::matrix::Mat3x4;
use pmath::ray::Ray;
use std::cmp::Eq;

//
// Scene
//

/// A handle to a geometry in the geometry pool. This is not a part of the scene itself yet.
#[derive(Copy, Clone, Debug)]
pub struct GeomPoolHandle {
    index: u32,
    embree_geom: embree::Geometry,
}

/// A handle to any geometry in the scene that a ray can intersect.
/// There exists one unique geometry handle for every object in a scene.
#[derive(Copy, Clone, Eq, Debug)]
pub struct GeomSceneHandle {
    geom_id: u32,
    inst_id: u32,
    // If the geometry should be treated as a light source. If it should,
    // then we have a light source handle.
    light_handle: Option<LightHandle>,
}

/// A handle to a light source in the scene.
#[derive(Copy, Clone, Eq, Debug)]
pub struct LightHandle {
    light_id: u32,
}

/// A handle to a specific material.
#[derive(Copy, Clone, Eq, Debug)]
pub struct MaterialHandle {
    material_id: u32,
}

/// A hanlde to a specific instance of a group.
#[derive(Copy, Clone, Eq, Debug)]
pub struct InstanceHandle {
    inst_id: u32,
}

/// A collection of geometries. These geometries are not part of the scene yet.
/// In order to perform instancing, the geometry must be part of the group (even
/// when instancing only one geometry).
pub struct GeomGroup {
    geometries: Vec<u32>,
    embree_scene: embree::Scene,
}

impl GeomGroup {
    /// Creates a new group.
    pub fn new() -> Self {
        Group {
            geometries: Vec::new(),
            embree_scene: embree::new_scene(),
        }
    }

    /// Adds a geometry to the group, returning
    pub fn add_geom(&mut self, geom: GeomPoolHandle) -> u32 {
        let geom_id = self.meshes.len() as u32;
        embree::attach_geometry_by_id(self.embree_scene, geom.embree_geom, geom_id);
        embree::commit_geometry(geom.embree_geom);
        self.meshes.push(geom.index);
        geom_id
    }
}

/// The instance id of the top level (used to index geometries that are not instanced).
const TOP_LEVEL_INST_ID: u32 = u32::max_value();

pub struct Scene {
    /// The mesh pool, which contains all of the mesh in a Scene.
    geom_pool: Vec<Box<dyn Geometry>>,
    /// The light pool, which contains all of the lights in a Scene.
    light_pool: Vec<Box<dyn Light>>,

    /// Contains all of the instances in a scene.
    instances: Vec<Instance>,
    /// Contains all of the top-level meshes in a scene.
    geometries: Vec<SceneGeom>,

    /// The embree pointer for the specific scene.
    embree_scene: embree::Scene,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            geom_pool: Vec::new(),
            light_pool: Vec::new(),
            instances: Vec::new(),
            geometries: Vec::new(),
            embree_scene: embree::new_scene(),
        }
    }

    /// Given a `GeomSceneHandle`, returns a reference to the geometry and material associated with the geometry.
    pub fn get_geom(&self, geom_handle: GeomSceneHandle) -> (&dyn Geometry, MaterialHandle) {
        if geom_handle.inst_id == TOP_LEVEL_INST_ID {
            let scene_geom = self.geometries[geom_ref.geom_id as usize];
            (
                &self.geom_pool[scene_geom.index],
                MaterialHandle {
                    material_id: scene_geom.material_id,
                },
            )
        } else {
            if (geom_handle.inst_id as usize) < self.geometries.len() {
                panic!("Invalid instance id.");
            }

            let instance = &self.instances[(geom_handle.inst_id as usize) - self.geometries.len()];
            let scene_geom = instance.geometries[geom_handle.geom_id as usize];
            (&self.geom_pool[scene_geom.index], scene_geom.material_id)
        }
    }

    /// Given a `LightHandle`, returns a reference to the light.
    pub fn get_light(&self, light_handle: LightHandle) -> &dyn Light {
        &self.light_pool[light_handle.light_id as usize]
    }

    /// Adds a mesh to the mesh pool of the scene, returning a handle to this specific geometry in the pool.
    pub fn add_to_geom_pool<T: Geometry>(&mut self, geom: T) -> GeomPoolHandle {
        let index = self.geom_pool.len() as u32;
        let embree_geom = geom.get_embree_geometry();
        self.geom_pool.push(Box::new(geom));
        GeomPoolHandle { index, embree_geom }
    }

    /// Adds a light to the scene, returning a global light index. Note that lights aren't instanced.
    /// If a light is associated with instanced geometry, have the light store a geom_id and inst_id.
    pub fn add_light<T: Light>(&mut self, light: T) -> LightHandle {
        let light_id = self.light_pool.len() as u32;
        self.light_pool.push(Box::new(light));
        LightHandle { light_id }
    }

    /// Sets the build quality to build the scene with.
    pub fn set_build_quality(&self, quality: embree::BuildQuality) {
        embree::set_scene_build_quality(self.embree_scene, quality);
    }

    /// Sets any flags to the scene (see `SceneFlags` enum).
    pub fn set_flags(&self, flags: embree::SceneFlags) {
        embree::set_scene_flags(self.embree_scene, flags);
    }

    /// After adding everything, this will build the top-level BVH:
    pub fn build_scene(&mut self) {
        embree::commit_scene(self.embree_scene);
        self.geom_pool.shrink_to_fit();
        self.light_pool.shrink_to_fit();
    }

    /// Adds a toplevel mesh and returns the geomID of that mesh.
    ///
    /// Adds a toplevel mesh with the given device and material id. Returning a geomID that is used
    /// to determine how reference it in the future. Note that these mesh should already have been
    /// transformed and CANNOT be animated.
    pub fn add_toplevel_geom(
        &mut self,
        geom: u32,
        material_handle: MaterialHandle,
    ) -> GeomSceneHandle {
        // Check if we are adding them too early:
        if !self.instances.is_empty() {
            panic!("Adding top level geometry after instance has already been added.")
        }

        // First create an rtc geometry of the mesh:
        let geom_id = self.geometries.len() as u32;
        embree::attach_geometry_by_id(self.embree_scene, geom.embree_geom, geom_id);
        embree::commit_geometry(geom.embree_geom);
        self.geometries.push(SceneGeom {
            index: geom.index,
            material_handle,
        });

        GeomSceneHandle {
            geom_id,
            inst_id: TOP_LEVEL_INST_ID,
            light_handle: None,
        }
    }

    /// Given a group, adds an instance of it in the scene.
    ///
    /// Adds an instance to the toplevel scene. Returns an `InstanceHandle` that can be used to get `GeomSceneHandle`s
    /// by calling `get_instance_geom_handle` with a specific `GeomGroupHandle`.
    pub fn add_group_instance(
        &mut self,
        group: &Group,
        materials: &[MaterialHandle],
        transform: Transf,
    ) -> InstanceHandle {
        let inst_id = (self.geometries.len() + self.instances.len()) as u32;

        // First we commit the scene:
        embree::commit_scene(group.embree_scene);

        let geom_ptr = embree::new_geometry(embree::GeometryType::Instance);
        embree::set_geometry_instance_scene(geom_ptr, group.embree_scene);
        embree::set_geometry_timestep_count(geom_ptr, 1);

        embree::attach_geometry_by_id(self.embree_scene, geom_ptr, inst_id);

        let mat = transform.get_frd().to_f32();
        embree::set_geometry_transform(
            geom_ptr,
            0,
            embree::Format::Float3x4RowMajor,
            &mat as *const Mat3x4<f32>,
        );

        embree::commit_geometry(geom_ptr);

        // Set the material id's for each mesh in the instance
        let mut geometries = Vec::with_capacity(group.meshes.len());
        for (&index, &material_handle) in group.meshes.iter().zip(materials.iter()) {
            geometries.push(SceneGeom {
                index,
                material_handle,
            })
        }

        self.instances.push(Instance {
            geometries,
            transform,
            embree_geom: geom_ptr,
        });

        InstanceHandle { inst_id }
    }

    /// Peforms an intersection, returning the interaction in world space.
    ///
    /// Given a ray, intersects the geometry and returns an interaction in the
    /// top-level scene space (aka world space).
    pub fn intersect(&self, ray: Ray<f64>) -> Option<GeomInteraction> {
        match embree::intersect(self.embree_scene, ray, 0, 0, 0) {
            Some((ray, hit)) => Some(self.calc_interaction(ray, hit)),
            _ => None,
        }
    }

    /// Performs an intersection test.
    ///
    /// Performs an intersection test. Returns true if intersection worked and false
    /// if it did not work.
    pub fn intersect_test(&self, ray: Ray<f64>) -> bool {
        embree::occluded(self.embree_scene, ray, 0, 0, 0)
    }

    /// Calculates the interaction given an RTCRayHit.
    fn calc_interaction(&self, ray: Ray<f64>, hit: embree::Hit<f64>) -> GeomInteraction {
        // Check if it hit a top-level geom or a bottom-level geom:
        let inst_id = hit.inst_id[0];
        if inst_id == embree::INVALID_GEOM_ID {
            // Get the top-level geom associated with the intersection:
            let scene_geom = &self.geometries[hit.geom_id as usize];
            // Get the specific primitive that we hit (the triangle):
            self.geom_pool[scene_geom.index as usize].calc_interaction(
                ray,
                hit,
                scene_geom.material_id,
            )
        } else {
            // Get an instance index. Because all instance geometry comes after the top-level geom,
            // we have to subtract the number of geom to get a local instance index into the vector:
            let inst_index = (inst_id as usize) - self.geometries.len();
            let instance = &self.instances[inst_index];
            // Get the mesh this instance was instancing:
            let scene_geom = &instance.geometries[hit.geom_id as usize];
            // Get the specific primitive that we hit (the triangle):
            let interaction = self.geom_pool[scene_geom.index as usize].calc_interaction(
                ray,
                hit,
                scene_geom.material_id,
            );
            // Don't forget to transform the interaction to world-space from the instance.
            instance.transform.geom_interaction(interaction)
        }
    }
}

/// A reference to a single mesh in the scene. It is
/// paired with a material id for that specific mesh.
#[derive(Clone, Copy, Debug)]
struct SceneGeom {
    index: u32,
    material_handle: MaterialHandle,
}

/// An instance of a mesh in the scene.
struct Instance {
    /// The collection of different meshes in the scene.
    geometries: Vec<SceneGeom>,
    /// Instance-to-world transformation.
    transform: Transf,
    /// The geometry the scene belongs to (as part of the top-level)
    embree_geom: embree::Geometry,
}
