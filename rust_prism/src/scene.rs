use crate::geometry::{GeomInteraction, Geometry};
use crate::light::Light;
use crate::transform::Transf;
use embree;
use pmath::matrix::Mat3x4;
use pmath::ray::Ray;

//
// Scene
//

/// Holds information about
#[derive(Copy, Clone, Debug)]
pub struct GeomRef {
    index: u32,
    embree_geom: embree::Geometry,
}

/// A group is a collection of mesh that can be instanced. They don't
/// exist during rendering, they are instead used during the construction
/// of a scene.
pub struct Group {
    geometries: Vec<u32>,
    embree_scene: embree::Scene,
}

impl Group {
    /// Creates a new group.
    pub fn new() -> Self {
        Group {
            geometries: Vec::new(),
            embree_scene: embree::new_scene(),
        }
    }

    /// Adds geometry to the group, returning the geometry ID that is local to this specific group.
    pub fn add_geom(&mut self, geom: GeomRef) -> u32 {
        let geom_id = self.meshes.len() as u32;
        embree::attach_geometry_by_id(self.embree_scene, mesh.embree_geom, geom_id);
        embree::commit_geometry(mesh.embree_geom);
        self.meshes.push(mesh.index);
        geom_id
    }
}

/// The instance id of the top level (used to index geometries that are not instanced).
const TOP_LEVEL_INST_ID: u32 = u32::max_value();

pub struct Scene {
    /// The mesh pool, which contains all of the mesh in a Scene. It also contains
    /// the embree geometry associated with it.
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

    /// Given a geometry id and an instance id, returns a reference to the geometry and material
    /// associated with the geometry.
    pub fn get_geom(&self, geom_id: u32, inst_id: u32) -> (&dyn Geometry, u32) {
        if inst_id == TOP_LEVEL_INST_ID {
            let scene_geom = self.geometries[geom_id as usize];
            (&self.geom_pool[scene_geom.index], scene_geom.material_id)
        } else {
            if (inst_id as usize) < self.geometries.len() {
                panic!("Invalid instance id provided");
            }

            let instance = &self.instances[(inst_id as usize) - self.geometries.len()];
            let scene_geom = instance.geometries[geom_id as usize];
            (&self.geom_pool[scene_geom.index], scene_geom.material_id)
        }
    }

    pub fn get_light(&self, light_id: u32) -> &dyn Light {
        &self.light_pool[light_id as usize]
    }

    /// Adds a mesh to the mesh pool of the scene, returning a geometry index.
    pub fn add_to_geom_pool<T: Geometry>(&mut self, geom: T) -> GeomRef {
        let index = self.geom_pool.len() as u32;
        let embree_geom = geom.get_embree_geometry();
        self.geom_pool.push(Box::new(geom));
        GeomRef { index, embree_geom }
    }

    /// Adds a light to the scene, returning a global light index. Note that lights aren't instanced.
    /// If a light is associated with instanced geometry, have the light store a geom_id and inst_id.
    pub fn add_light<T: Light>(&mut self, light: T) -> u32 {
        let index = self.light_pool.len() as u32;
        self.light_pool.push(Box::new(light));
        index
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
    pub fn add_toplevel_geom(&mut self, geom: u32, material_id: u32) -> u32 {
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
            material_id,
        });
        geom_id
    }

    /// Given a group, adds an instance of it in the scene.
    ///
    /// Adds an instance to the toplevel scene. Returns the instID (geomID in the top-level scene).
    /// Pass in the material_id for each of the group mesh in the group. Must be the same length as
    /// the number of mesh in the group. Ordering is based on geom_id returned by add_group_mesh.
    pub fn add_group_instance(
        &mut self,
        group: &Group,
        material_ids: &[u32],
        transform: Transf,
    ) -> u32 {
        let geom_id = (self.geometries.len() + self.instances.len()) as u32;

        // First we commit the scene:
        embree::commit_scene(group.embree_scene);

        let geom_ptr = embree::new_geometry(embree::GeometryType::Instance);
        embree::set_geometry_instance_scene(geom_ptr, group.embree_scene);
        embree::set_geometry_timestep_count(geom_ptr, 1);

        embree::attach_geometry_by_id(self.embree_scene, geom_ptr, geom_id);

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
        for (&index, &material_id) in group.meshes.iter().zip(material_ids.iter()) {
            geometries.push(SceneGeom { index, material_id })
        }

        self.instances.push(Instance {
            geometries,
            transform,
            embree_geom: geom_ptr,
        });

        geom_id
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
    material_id: u32,
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
