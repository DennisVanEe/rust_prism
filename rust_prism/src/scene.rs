use crate::embree::{
    BuildQuality, Format, GeometryPtr, GeometryType, SceneFlags, ScenePtr, DEVICE,
};
use crate::math::matrix::Mat3x4;
use crate::math::ray::Ray;
use crate::mesh::{Interaction, Mesh, MeshRef};
use crate::transform::Transf;
use bumpalo::Bump;
use lazy_static::lazy_static;
use std::mem::{self, MaybeUninit};
use std::os::raw;
use std::sync::Mutex;

//
// Mesh Pool
//

struct MeshPool {
    bump: Bump,
}
// We have to implement this otherwise lazy_static doesn't work.
unsafe impl Sync for MeshPool {}

const NUM_MESH_PER_CHUNK: usize = 16;

lazy_static! {
    // The actual mesh pool (not to be accessed directly)
    static ref MESH_POOL: MeshPool = {
        MeshPool {
            bump: Bump::with_capacity(NUM_MESH_PER_CHUNK * mem::size_of::<Mesh>()),
        }
    };
    // This is what actually gets accessed (and we need it to be locked)
    static ref LOCKED_MESH_POOL: Mutex<&'static MeshPool> = {
        Mutex::new(&MESH_POOL)
    };
}

/// Given a mesh, allocates it onto the heap. This is thread safe and will allow multiple
/// threads to perform this.
pub fn allocate_mesh(mesh: Mesh) -> MeshRef<'static> {
    let mesh_pool = LOCKED_MESH_POOL.lock().unwrap();
    mesh_pool.bump.alloc_with(|| mesh).get_ref()
}

//
// Scene
//

/// A group is a collection of mesh that can be instanced.
pub struct Group {
    meshes: Vec<MeshRef<'static>>,
    /// The scene that this mesh belongs to.
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

    /// Adds a mesh to the group. Returns the geometry id (local to the group).
    pub fn add_mesh(&mut self, mesh: MeshRef<'static>) -> u32 {
        let geom_ptr = mesh.get_embree_geometry();
        let geom_id = self.meshes.len() as u32;
        DEVICE.attach_geometry_by_id(self.embree_scene, geom_ptr, geom_id);
        DEVICE.commit_geometry(geom_ptr);
        self.meshes.push(mesh);
        geom_id
    }
}

pub struct Scene {
    /// A collection of the different instances.
    instances: Vec<Instance>,
    /// A collection of top-level meshes in the scene.
    meshes: Vec<SceneMesh>,
    /// A pointer to the embree scene.
    embree_scene: ScenePtr,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            instances: Vec::new(),
            meshes: Vec::new(),
            embree_scene: DEVICE.new_scene(),
        }
    }

    pub fn set_build_quality(&self, quality: BuildQuality) {
        DEVICE.set_scene_build_quality(self.embree_scene, quality);
    }

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
    pub fn add_toplevel_mesh(&mut self, mesh: MeshRef<'static>, material_id: u32) -> u32 {
        // First create an rtc geometry of the mesh:
        let geom_ptr = mesh.get_embree_geometry();
        let geom_id = self.meshes.len() as u32;
        DEVICE.attach_geometry_by_id(self.embree_scene, geom_ptr, geom_id);
        DEVICE.commit_geometry(geom_ptr);
        self.meshes.push(SceneMesh { mesh, material_id });
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
        for (&mesh, &material_id) in group.meshes.iter().zip(material_ids.iter()) {
            meshes.push(SceneMesh { mesh, material_id })
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
            let scene_mesh = unsafe { self.meshes.get_unchecked(rayhit.hit.geomID as usize) };
            // Get the specific primitive that we hit (the triangle):
            let triangle = unsafe {
                scene_mesh
                    .mesh
                    .triangles
                    .get_unchecked(rayhit.hit.primID as usize)
            };
            triangle.calc_interaction(rayhit, scene_mesh.mesh, scene_mesh.material_id)
        } else {
            // Get an instance index. Because all instance geometry comes after the top-level meshes,
            // we have to subtract the number of meshes to get a local instance index into the vector:
            let inst_index = (inst_id as usize) - self.meshes.len();
            let instance = unsafe { self.instances.get_unchecked(inst_index) };
            // Get the mesh this instance was instancing:
            let scene_mesh = unsafe { instance.meshes.get_unchecked(rayhit.hit.geomID as usize) };
            // Get the specific primitive that we hit (the triangle):
            let triangle = unsafe {
                scene_mesh
                    .mesh
                    .triangles
                    .get_unchecked(rayhit.hit.primID as usize)
            };
            let interaction =
                triangle.calc_interaction(rayhit, scene_mesh.mesh, scene_mesh.material_id);
            // Don't forget to transform the interaction to world-space from the instance.
            instance.transform.interaction(interaction)
        }
    }
}

/// A reference to a single mesh in the scene. It is
/// paired with a material id for that specific mesh.
struct SceneMesh {
    mesh: MeshRef<'static>,
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
