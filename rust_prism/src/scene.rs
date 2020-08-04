use crate::geometry::{GeomInteraction, Geometry};
use crate::light::Light;
use crate::transform::Transf;
use embree;
use pmath::matrix::Mat3x4;
use pmath::ray::Ray;

//
// Scene
//

/// A simple structure that is used to refer to a mesh once it has been added to the Scene's mesh pool.
#[derive(Clone, Copy, Debug)]
pub struct GeomRef {
    index: u32,
    embree_geom: embree::Geometry,
}

/// A simple structure that is used to refer to a light once it has been added to the Light's mesh pool.
#[derive(Clone, Copy, Debug)]
pub struct LightRef {
    index: u32,
}

/// A group is a collection of mesh that can be instanced.
pub struct Group {
    meshes: Vec<u32>,
    embree_scene: embree::Scene,
}

impl Group {
    /// Creates a new group.
    pub fn new() -> Self {
        Group {
            meshes: Vec::new(),
            embree_scene: embree::new_scene(),
        }
    }

    /// Adds a mesh to the group (as a MeshID). Returns the geometry id (local to the group).
    pub fn add_geom(&mut self, mesh: GeomRef) -> u32 {
        let geom_id = self.meshes.len() as u32;
        embree::attach_geometry_by_id(self.embree_scene, mesh.embree_geom, geom_id);
        embree::commit_geometry(mesh.embree_geom);
        self.meshes.push(mesh.index);
        geom_id
    }
}

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

    /// Adds a mesh to the mesh pool of the scene.
    pub fn add_to_geom_pool<T: Geometry>(&mut self, geom: T) -> GeomRef {
        let index = self.geom_pool.len() as u32;
        let embree_geom = geom.get_embree_geometry();
        self.geom_pool.push(Box::new(geom));
        GeomRef { index, embree_geom }
    }

    /// Adds a light to the light pool of the scene.
    pub fn add_to_light_pool<T: Light>(&mut self, light: T) -> LightRef {
        let index = self.light_pool.len() as u32;
        self.light_pool.push(Box::new(light));
        LightRef { index }
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
    pub fn build_scene(&self) {
        embree::commit_scene(self.embree_scene);
    }

    /// Adds a toplevel mesh and returns the geomID of that mesh.
    ///
    /// Adds a toplevel mesh with the given device and material id. Returning a geomID that is used
    /// to determine how reference it in the future. Note that these mesh should already have been
    /// transformed and CANNOT be animated.
    pub fn add_toplevel_geom(&mut self, geom: GeomRef, material_id: u32) -> u32 {
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
