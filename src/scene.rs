use crate::device::Device;
use crate::math::matrix::Mat3x4;
use crate::math::ray::Ray;
use crate::mesh::{Interaction, Mesh};
use crate::transform::Transform;
use bumpalo::Bump;
use embree;
use simple_error::SimpleResult;
use std::mem::MaybeUninit;
use std::os::raw;

struct SceneMesh<'a> {
    mesh: &'a Mesh,
    material_id: u32,
}

pub struct Instance<'a> {
    meshes: Vec<SceneMesh<'a>>,
    transform: Transform,             // instance-to-world transformation
    embree_geom: embree::RTCGeometry, // The geometry the scene belongs to (as part of the top-level)
}

// A group is a collection of mesh that can be instanced:
pub struct Group<'a> {
    meshes: Vec<&'a Mesh>,
    embree_scene: embree::RTCScene, // The scene the instance belongs to
}

impl<'a> Group<'a> {
    pub fn new(device: &Device) -> SimpleResult<Self> {
        Ok(Group {
            meshes: Vec::new(),
            embree_scene: device.new_scene()?,
        })
    }
}

pub struct Scene<'a> {
    // All of the mesh reside in here:
    mesh_pool: Bump,
    instances: Vec<Instance<'a>>,
    meshes: Vec<SceneMesh<'a>>,

    embree_scene: embree::RTCScene,
}

impl<'a> Scene<'a> {
    const NUM_MESH_PER_CHUNK: usize = 16; // Make bigger later with bigger scenes

    pub fn new(device: &Device) -> SimpleResult<Self> {
        let embree_scene = device.new_scene()?;

        Ok(Scene {
            mesh_pool: Bump::with_capacity(Self::NUM_MESH_PER_CHUNK),
            instances: Vec::new(),
            meshes: Vec::new(),
            embree_scene,
        })
    }

    // Adds a toplevel mesh and returns the geomID of that mesh:
    pub fn add_toplevel_mesh(
        &mut self,
        device: &Device,
        mesh: &'a Mesh,
        material_id: u32,
    ) -> SimpleResult<u32> {
        // First create an rtc geometry of the mesh:
        let rtcgeom = mesh.create_rtcgeom(device)?;
        let geom_id = self.meshes.len() as u32;
        unsafe {
            embree::rtcAttachGeometryByID(self.embree_scene, rtcgeom, geom_id);
        }
        device.error()?;
        unsafe {
            // TODO: figure out if this commit is needed here
            embree::rtcCommitGeometry(rtcgeom);
        }
        device.error()?;

        self.meshes.push(SceneMesh { mesh, material_id });

        Ok(geom_id)
    }

    // Adds a bottomlevel mesh to the provided group and returns the geometry id specific to that mesh:
    pub fn add_group_mesh(
        &mut self,
        group: &mut Group<'a>,
        mesh: &'a Mesh,
        device: &Device,
    ) -> SimpleResult<u32> {
        let rtcgeom = mesh.create_rtcgeom(device)?;
        let geom_id = group.meshes.len() as u32;
        unsafe {
            embree::rtcAttachGeometryByID(group.embree_scene, rtcgeom, geom_id);
        }
        device.error()?;
        group.meshes.push(mesh);
        Ok(geom_id)
    }

    // Adds an instance to the toplevel scene. Returns the instID (geomID in the top-level scene).
    // Pass in the material_id for each of the group mesh in the group. Must be the same length as
    // the number of mesh in the group. Ordering is based on geom_id returned by add_group_mesh.
    pub fn add_group_instance(
        &mut self,
        group: &Group<'a>,
        material_ids: &[u32],
        transform: Transform,
        device: &Device,
    ) -> SimpleResult<u32> {
        let geom_id = (self.meshes.len() + self.instances.len()) as u32;

        // We have to do this first:
        unsafe {
            embree::rtcCommitScene(group.embree_scene);
        }
        device.error()?;

        let rtcgeom = device.new_geometry(embree::RTCGeometryType_RTC_GEOMETRY_TYPE_INSTANCE)?;
        unsafe {
            embree::rtcSetGeometryInstancedScene(rtcgeom, self.embree_scene);
        }
        device.error()?;
        unsafe {
            embree::rtcSetGeometryTimeStepCount(rtcgeom, 1); // No motion-blurr support yet
        }
        device.error()?;
        // Apply the transformation:
        let xmf = transform.get_mat().to_f32();
        unsafe {
            embree::rtcSetGeometryTransform(
                rtcgeom,
                0,
                embree::RTCFormat_RTC_FORMAT_FLOAT3X4_ROW_MAJOR,
                (&xmf as *const Mat3x4<f32>) as *const raw::c_void,
            );
        }
        device.error()?;
        unsafe {
            embree::rtcCommitGeometry(rtcgeom);
        }
        device.error()?;
        unsafe {
            embree::rtcAttachGeometryByID(self.embree_scene, rtcgeom, geom_id);
        }
        device.error()?;

        // Now we create the instance information ourselves:
        let mut meshes = Vec::with_capacity(group.meshes.len());
        for (mesh, &material_id) in group.meshes.iter().zip(material_ids.iter()) {
            meshes.push(SceneMesh { mesh, material_id })
        }

        self.instances.push(Instance {
            meshes,
            transform,
            embree_geom: rtcgeom,
        });

        Ok(geom_id)
    }

    // Adds a mesh to the pool and returns a reference. Use this reference in the future
    // everytime you want to write stuff and whatnot.
    pub fn add_mesh(&mut self, mesh: Mesh) -> &Mesh {
        self.mesh_pool.alloc(mesh)
    }

    // Given a ray, intersects the geometry and returns an interaction in the
    // top-level scene space (aka world space).
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
                self.embree_scene,
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

    // Performs an intersection test. Returns true if intersection worked and false
    // if it did not work.
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
                self.embree_scene,
                &mut context as *mut embree::RTCIntersectContext,
                &mut rayhit as *mut embree::RTCRay,
            );
        }
        // If a hit was registered, set it to negative infinity
        return rayhit.tfar == f32::NEG_INFINITY;
    }

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
            instance.transform.interaction(interaction)
        }
    }
}
