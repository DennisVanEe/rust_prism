use crate::embree::{
    BuildQuality, Format, GeometryPtr, GeometryType, SceneFlags, ScenePtr, DEVICE,
};
use crate::light::Light;
use crate::math::matrix::Mat3x4;
use crate::math::ray::Ray;
use crate::mesh::{Interaction, Mesh};
use crate::transform::Transf;
use std::mem::MaybeUninit;
use std::os::raw;

//
// Scene
//

/// A simple structure that is used to refer to a mesh once it has been added to the Scene's mesh pool.
#[derive(Clone, Copy, Debug)]
pub struct MeshRef {
    index: u32,
    embree_geom: GeometryPtr,
}

/// A simple structure that is used to refer to a light once it has been added to the Light's mesh pool.
#[derive(Clone, Copy, Debug)]
pub struct LightRef {
    index: u32,
}

/// A group is a collection of mesh that can be instanced.
pub struct Group {
    meshes: Vec<u32>,
    embree_scene: ScenePtr,
}

impl Group {
    /// Creates a new group.
    pub fn new() -> Self {
        Group {
            meshes: Vec::new(),
            embree_scene: DEVICE.new_scene(),
        }
    }

    /// Adds a mesh to the group (as a MeshID). Returns the geometry id (local to the group).
    pub fn add_mesh(&mut self, mesh: MeshRef) -> u32 {
        let geom_ptr = mesh.embree_geom;
        let geom_id = self.meshes.len() as u32;
        DEVICE.attach_geometry_by_id(self.embree_scene, geom_ptr, geom_id);
        DEVICE.commit_geometry(geom_ptr);
        self.meshes.push(mesh.index);
        geom_id
    }
}

pub struct Scene {
    /// The mesh pool, which contains all of the mesh in a Scene.
    mesh_pool: Vec<Mesh>,
    /// The light pool, which contains all of the lights in a Scene.
    light_pool: Vec<Box<dyn Light>>,

    /// Contains all of the instances in a scene.
    instances: Vec<Instance>,
    /// Contains all of the top-level meshes in a scene.
    meshes: Vec<SceneMesh>,

    /// The embree pointer for the specific scene.
    embree_scene: ScenePtr,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            mesh_pool: Vec::new(),
            light_pool: Vec::new(),
            instances: Vec::new(),
            meshes: Vec::new(),
            embree_scene: DEVICE.new_scene(),
        }
    }

    /// Adds a mesh to the mesh pool of the scene.
    pub fn add_to_mesh_pool(&mut self, mesh: Mesh) -> MeshRef {
        let index = self.mesh_pool.len() as u32;
        let embree_geom = mesh.get_embree_geom();
        self.mesh_pool.push(mesh);
        MeshRef { index, embree_geom }
    }

    /// Adds a light to the light pool of the scene.
    pub fn add_to_light_pool<T: Light>(&mut self, light: T) -> LightRef {
        // Create a Box of the light:
        let light_box = Box::new(light);
        let index = self.light_pool.len();
        self.light_pool.push(light_box);
        LightRef { index }
    }

    /// Sets the build quality to build the scene with.
    pub fn set_build_quality(&self, quality: BuildQuality) {
        DEVICE.set_scene_build_quality(self.embree_scene, quality);
    }

    /// Sets any flags to the scene (see `SceneFlags` enum).
    pub fn set_flags(&self, flags: SceneFlags) {
        DEVICE.set_scene_flags(self.embree_scene, flags);
    }

    /// After adding everything, this will build the top-level BVH:
    pub fn build_scene(&self) {
        DEVICE.commit_scene(self.embree_scene);
    }

    /// Adds a toplevel mesh and returns the geomID of that mesh.
    ///
    /// Adds a toplevel mesh with the given device and material id. Returning a geomID that is used
    /// to determine how reference it in the future. Note that these mesh should already have been
    /// transformed and CANNOT be animated.
    pub fn add_toplevel_mesh(&mut self, mesh: MeshRef, material_id: u32) -> u32 {
        // First create an rtc geometry of the mesh:
        let geom_ptr = mesh.embree_geom;
        let geom_id = self.meshes.len() as u32;
        DEVICE.attach_geometry_by_id(self.embree_scene, geom_ptr, geom_id);
        DEVICE.commit_geometry(geom_ptr);
        self.meshes.push(SceneMesh {
            index: mesh.index,
            material_id,
        });
        geom_id
    }

    pub fn add_toplevel_light(&mut self, ) -> u32 {
        
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
        let geom_id = (self.meshes.len() + self.instances.len()) as u32;

        // First we commit the scene:
        DEVICE.commit_scene(group.embree_scene);

        let geom_ptr = DEVICE.new_geometry(GeometryType::Instance);
        DEVICE.set_geometry_instance_scene(geom_ptr, group.embree_scene);
        DEVICE.set_geometry_timestep_count(geom_ptr, 1);

        DEVICE.attach_geometry_by_id(self.embree_scene, geom_ptr, geom_id);

        let mat = transform.get_frd().to_f32();
        DEVICE.set_geometry_transform(
            geom_ptr,
            0,
            Format::Float3x4RowMajor,
            (&mat as *const Mat3x4<f32>) as *const raw::c_void,
        );

        DEVICE.commit_geometry(geom_ptr);

        // Set the material id's for each mesh in the instance
        let mut meshes = Vec::with_capacity(group.meshes.len());
        for (&index, &material_id) in group.meshes.iter().zip(material_ids.iter()) {
            meshes.push(SceneMesh { index, material_id })
        }

        self.instances.push(Instance {
            meshes,
            transform,
            embree_geom: geom_ptr,
        });

        geom_id
    }

    /// Peforms an intersection, returning the interaction in world space.
    ///
    /// Given a ray, intersects the geometry and returns an interaction in the
    /// top-level scene space (aka world space).
    pub fn intersect(&self, ray: Ray<f64>) -> Option<Interaction> {
        let mut context = unsafe { MaybeUninit::uninit().assume_init() };
        embree::rtcInitIntersectContext(&mut context);
        let mut rayhit = embree::RTCRayHit {
            ray: embree::RTCRay {
                org_x: ray.org.x as f32,
                org_y: ray.org.y as f32,
                org_z: ray.org.z as f32,
                tnear: ray.t_near as f32,
                dir_x: ray.dir.x as f32,
                dir_y: ray.dir.y as f32,
                dir_z: ray.dir.z as f32,
                time: ray.time as f32,
                tfar: ray.t_far as f32,
                // This isn't utilized yet:
                mask: 0,
                id: 0,
                flags: 0,
            },
            hit: unsafe { MaybeUninit::uninit().assume_init() },
        };
        rayhit.hit.geomID = embree::RTC_INVALID_GEOMETRY_ID;
        unsafe {
            embree::rtcIntersect1(
                self.embree_scene.get_raw(),
                &mut context as *mut embree::RTCIntersectContext,
                &mut rayhit as *mut embree::RTCRayHit,
            );
        }
        // Check for intersection:
        if rayhit.hit.geomID == embree::RTC_INVALID_GEOMETRY_ID {
            return None;
        }
        Some(self.calc_interaction(rayhit))
    }

    /// Performs an intersection test.
    ///
    /// Performs an intersection test. Returns true if intersection worked and false
    /// if it did not work.
    pub fn intersect_test(&self, ray: Ray<f64>) -> bool {
        let mut context = unsafe { MaybeUninit::uninit().assume_init() };
        embree::rtcInitIntersectContext(&mut context);
        let mut rayhit = embree::RTCRay {
            org_x: ray.org.x as f32,
            org_y: ray.org.y as f32,
            org_z: ray.org.z as f32,
            tnear: ray.t_near as f32,
            dir_x: ray.dir.x as f32,
            dir_y: ray.dir.y as f32,
            dir_z: ray.dir.z as f32,
            time: ray.time as f32,
            tfar: ray.t_far as f32,
            // This isn't utilized yet:
            mask: 0,
            id: 0,
            flags: 0,
        };
        unsafe {
            embree::rtcOccluded1(
                self.embree_scene.get_raw(),
                &mut context as *mut embree::RTCIntersectContext,
                &mut rayhit as *mut embree::RTCRay,
            );
        }
        // If a hit was registered, set it to negative infinity
        return rayhit.tfar == f32::NEG_INFINITY;
    }

    /// Calculates the interaction given an RTCRayHit.
    fn calc_interaction(&self, rayhit: embree::RTCRayHit) -> Interaction {
        // Check if it hit a top-level mesh or a bottom-level mesh:
        let inst_id = rayhit.hit.instID[0];
        if inst_id == embree::RTC_INVALID_GEOMETRY_ID {
            // Get the top-level mesh associated with the intersection:
            let scene_mesh = &self.meshes[rayhit.hit.geomID as usize];
            // Get the specific primitive that we hit (the triangle):
            let mesh = &self.mesh_pool[scene_mesh.index as usize];
            let triangle = mesh.triangles[rayhit.hit.primID as usize];
            triangle.calc_interaction(rayhit, mesh, scene_mesh.material_id)
        } else {
            // Get an instance index. Because all instance geometry comes after the top-level meshes,
            // we have to subtract the number of meshes to get a local instance index into the vector:
            let inst_index = (inst_id as usize) - self.meshes.len();
            let instance = &self.instances[inst_index];
            // Get the mesh this instance was instancing:
            let scene_mesh = &instance.meshes[rayhit.hit.geomID as usize];
            // Get the specific primitive that we hit (the triangle):
            let mesh = &self.mesh_pool[scene_mesh.index as usize];
            let triangle = mesh.triangles[rayhit.hit.primID as usize];
            let interaction = triangle.calc_interaction(rayhit, mesh, scene_mesh.material_id);
            // Don't forget to transform the interaction to world-space from the instance.
            instance.transform.interaction(interaction)
        }
    }
}

/// A reference to a single mesh in the scene. It is
/// paired with a material id for that specific mesh.
struct SceneMesh {
    index: u32,
    material_id: u32,
}

/// An instance of a mesh in the scene.
struct Instance {
    /// The collection of different meshes in the scene.
    meshes: Vec<SceneMesh>,
    /// Instance-to-world transformation.
    transform: Transf,
    /// The geometry the scene belongs to (as part of the top-level)
    embree_geom: GeometryPtr,
}

impl Drop for Instance {
    fn drop(&mut self) {
        DEVICE.release_geometry(self.embree_geom);
    }
}
