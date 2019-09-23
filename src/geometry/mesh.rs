// This file stores the implementation of Mesh, which is just a collection of face
// indices and vertex positions. It also describes the triangle intersection algorithm
// used by PRISM (basically pbrt's version).

use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::util::{align, coord_system};
use crate::math::vector::{Vec2, Vec3, Vec3Perm};
use crate::transform::Transform;

use order_stat::kth_by;
use partition::partition;

use std::cell::Cell;
use std::mem::MaybeUninit;

// A Mesh is not a geometric object. Instead it just stores a collection
// of points.
#[derive(Clone, Debug)]
pub struct Mesh {
    tris: Vec<Triangle>,
    data: Vec<f32>,

    // If the value is 0xFF, it exists, if it is 0x00, it doesn't:
    has_nrm: u8,
    has_tan: u8,
    // We don't need 0xFF stuff for has_uvs:
    has_uvs: bool,

    // number of properties per vertex
    // (in terms of the number of floats):
    num_prop: u8,
    num_vert: u32,
}

// Mesh access is done through u32 values to save on storage:
impl Mesh {
    // Important thing to note, if has_tan = true, then has_nrm must be true as well. If this is not
    // the case, bad things will happen....
    pub fn new(
        tris: Vec<Triangle>,
        data: Vec<f32>,
        has_nrm: bool,
        has_tan: bool,
        has_uvs: bool,
    ) -> Self {
        // Make sure that, if has_tan is true, has_nrm is also true:
        debug_assert!(has_nrm || (!has_nrm && !has_tan));

        let has_nrm = if has_nrm { 0xff } else { 0x00 };
        let has_tan = if has_tan { 0xff } else { 0x00 };

        // Performs a logical right shift because it's unsigned:
        // Always add 3 (the position information, which must always be present):
        let num_prop =
            3 + ((has_nrm >> 7) * 3) + ((has_tan >> 7) * 3) + if has_uvs { 2 } else { 0 };
        // Down casts value, we are guaranteed it's a multiple:
        let num_vert = (data.len() / (num_prop as usize)) as u32;

        Mesh {
            tris,
            data,
            has_nrm,
            has_tan,
            has_uvs,
            num_prop,
            num_vert,
        }
    }

    pub fn update_tris(&mut self, tris: Vec<Triangle>) {
        self.tris = tris;
    }

    pub fn update_data(&mut self, data: Vec<f32>, has_nrm: bool, has_tan: bool, has_uvs: bool) {
        // Make sure that, if has_tan is true, has_nrm is also true:
        debug_assert!(has_nrm || (!has_nrm && !has_tan));

        let has_nrm = if has_nrm { 0xffu8 } else { 0x00u8 };
        let has_tan = if has_tan { 0xffu8 } else { 0x00u8 };

        // Performs a logical right shift because it's unsigned:
        // Always add 3 (the position information, which must always be present):
        let num_prop = 3u8
            + ((has_nrm >> 7u8) * 3u8)
            + ((has_tan >> 7u8) * 3u8)
            + if has_uvs { 2u8 } else { 0u8 };
        // Down casts value, we are guaranteed it's a multiple:
        let num_vert = (data.len() / (num_prop as usize)) as u32;

        self.data = data;
        self.has_nrm = has_nrm;
        self.has_tan = has_tan;
        self.has_uvs = has_uvs;
        self.num_prop = num_prop;
        self.num_vert = num_vert;
    }

    // Raw access to the data (can't modify):
    pub fn get_tri_raw(&self) -> &Vec<Triangle> {
        &self.tris
    }

    pub fn get_data_raw(&self) -> &Vec<f32> {
        &self.data
    }

    pub fn num_tris(&self) -> u32 {
        self.tris.len() as u32
    }

    // Returns a single triangle:
    pub fn get_tri(&self, index: u32) -> Triangle {
        unsafe { *self.tris.get_unchecked(index as usize) }
    }

    pub fn set_tri(&mut self, index: u32, tri: Triangle) {
        unsafe {
            *self.tris.get_unchecked_mut(index as usize) = tri;
        }
    }

    pub fn num_vert(&self) -> u32 {
        self.num_vert
    }

    pub fn has_nrm(&self) -> bool {
        self.has_nrm == 0xffu8
    }

    pub fn has_tan(&self) -> bool {
        self.has_tan == 0xffu8
    }

    pub fn has_uvs(&self) -> bool {
        self.has_uvs
    }

    pub fn get_surface_area(&self) -> f64 {
        self.tris.iter().fold(0., |area, tri| area + tri.area(self))
    }

    // The functions below are unsafe because there is no check for
    // whether or not the property exists and tha the triangle index
    // given is present:

    pub unsafe fn get_pos(&self, index: u32) -> Vec3<f64> {
        let index = (self.num_prop as usize) * (index as usize);
        Vec3 {
            x: *self.data.get_unchecked(index + 0) as f64,
            y: *self.data.get_unchecked(index + 1) as f64,
            z: *self.data.get_unchecked(index + 2) as f64,
        }
    }

    // make sure you have normals first...
    pub unsafe fn get_nrm(&self, index: u32) -> Vec3<f64> {
        debug_assert!(self.has_nrm());

        // If we have normal information, it'll always follow the position:
        let index = (self.num_prop as usize) * (index as usize) + 3;
        Vec3 {
            x: *self.data.get_unchecked(index + 0) as f64,
            y: *self.data.get_unchecked(index + 1) as f64,
            z: *self.data.get_unchecked(index + 2) as f64,
        }
    }

    pub unsafe fn get_tan(&self, index: u32) -> Vec3<f64> {
        debug_assert!(self.has_tan());

        // If we have tangent information, it will always follow normal position:
        let index = (self.num_prop as usize) * (index as usize) + 6;
        Vec3 {
            x: *self.data.get_unchecked(index + 0) as f64,
            y: *self.data.get_unchecked(index + 1) as f64,
            z: *self.data.get_unchecked(index + 2) as f64,
        }
    }

    pub unsafe fn get_uvs(&self, index: u32) -> Vec2<f64> {
        debug_assert!(self.has_uvs());

        // Here we have to do a bit more work, because UVs cana technically belong anywhere:
        let index = (self.num_prop as usize) * (index as usize)
            + 3
            + ((self.has_nrm & 3) as usize)
            + ((self.has_tan & 3) as usize);
        Vec2 {
            x: *self.data.get_unchecked(index + 0) as f64,
            y: *self.data.get_unchecked(index + 1) as f64,
        }
    }

    pub unsafe fn set_pos(&mut self, index: u32, vec: Vec3<f64>) {
        let index = (self.num_prop as usize) * (index as usize);
        *self.data.get_unchecked_mut(index + 0) = vec.x as f32;
        *self.data.get_unchecked_mut(index + 1) = vec.y as f32;
        *self.data.get_unchecked_mut(index + 2) = vec.z as f32;
    }

    pub unsafe fn set_nrm(&mut self, index: u32, vec: Vec3<f64>) {
        debug_assert!(self.has_nrm());

        let index = (self.num_prop as usize) * (index as usize) + 3;
        *self.data.get_unchecked_mut(index + 0) = vec.x as f32;
        *self.data.get_unchecked_mut(index + 1) = vec.y as f32;
        *self.data.get_unchecked_mut(index + 2) = vec.z as f32;
    }

    pub unsafe fn set_tan(&mut self, index: u32, vec: Vec3<f32>) {
        debug_assert!(self.has_tan());

        let index = (self.num_prop as usize) * (index as usize) + 6;
        *self.data.get_unchecked_mut(index + 0) = vec.x as f32;
        *self.data.get_unchecked_mut(index + 1) = vec.y as f32;
        *self.data.get_unchecked_mut(index + 2) = vec.z as f32;
    }

    pub unsafe fn set_uvs(&mut self, index: u32, vec: Vec2<f64>) {
        debug_assert!(self.has_uvs());

        let index = (self.num_prop as usize) * (index as usize)
            + 3
            + ((self.has_nrm & 3) as usize)
            + ((self.has_tan & 3) as usize);
        *self.data.get_unchecked_mut(index + 0usize) = vec.x as f32;
        *self.data.get_unchecked_mut(index + 1usize) = vec.y as f32;
    }
}

// Stores extra information used to speed up ray intersection calculations:
#[derive(Clone, Copy)]
pub struct RayIntInfo {
    shear: Vec3<f64>,
    perm_dir: Vec3<f64>,
    perm: Vec3Perm,
}

// Given a ray, calculates the ray intersection information used for
// efficient ray-triangle intersection.
pub fn calc_rayintinfo(ray: Ray<f64>) -> RayIntInfo {
    let z = ray.dir.abs().max_dim();
    let x = if z == 2 { 0 } else { z + 1 };
    let y = if x == 2 { 0 } else { x + 1 };

    let perm = Vec3Perm::new(x, y, z);
    let perm_dir = ray.dir.permute(perm);

    let inv_perm_dir_z = 1. / perm_dir.z;
    let shear = Vec3 {
        x: -perm_dir.x * inv_perm_dir_z,
        y: -perm_dir.y * inv_perm_dir_z,
        z: inv_perm_dir_z,
    };

    RayIntInfo {
        shear,
        perm_dir,
        perm,
    }
}

// The triangle itself doesn't store the mesh, it just stores the indices:
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    // ray:      the ray that will intersect the triangle
    // max_time: the maximum time that we will be considering (can prune early triangles)
    // int_info: intersection info used to accelerate intersections
    // mesh:     the mesh of the triangle that intersects it
    // return    option with time where the intersection occurs.
    pub fn intersect_test(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        int_info: RayIntInfo,
        mesh: &Mesh,
    ) -> bool {
        let poss = self.get_poss(mesh);

        let pt = [poss[0] - ray.org, poss[1] - ray.org, poss[2] - ray.org];
        let pt = [
            pt[0].permute(int_info.perm),
            pt[1].permute(int_info.perm),
            pt[2].permute(int_info.perm),
        ];
        let pt = [
            Vec3 {
                x: int_info.shear.x * pt[0].z + pt[0].x,
                y: int_info.shear.y * pt[0].z + pt[0].y,
                z: pt[0].z,
            },
            Vec3 {
                x: int_info.shear.x * pt[1].z + pt[1].x,
                y: int_info.shear.y * pt[1].z + pt[1].y,
                z: pt[1].z,
            },
            Vec3 {
                x: int_info.shear.x * pt[2].z + pt[2].x,
                y: int_info.shear.y * pt[2].z + pt[2].y,
                z: pt[2].z,
            },
        ];

        // Calculate the edge function results:
        let e = [
            pt[1].x * pt[2].y - pt[1].y * pt[2].x,
            pt[2].x * pt[0].y - pt[2].y * pt[0].x,
            pt[0].x * pt[1].y - pt[0].y * pt[1].x,
        ];

        // Check if our ray lands outside of the edges of the triangle:
        if (e[0] < 0. || e[1] < 0. || e[2] < 0.) && (e[0] > 0. || e[1] > 0. || e[2] > 0.) {
            return false;
        };

        let sum_e = e[0] + e[1] + e[2];
        // Checks if it's a degenerate triangle:
        if sum_e == 0. {
            return false;
        };

        // Now we finish transforming the z value:
        let pt = [
            Vec3 {
                x: pt[0].x,
                y: pt[0].y,
                z: pt[0].z * int_info.shear.z,
            },
            Vec3 {
                x: pt[1].x,
                y: pt[1].y,
                z: pt[1].z * int_info.shear.z,
            },
            Vec3 {
                x: pt[2].x,
                y: pt[2].y,
                z: pt[2].z * int_info.shear.z,
            },
        ];

        let time_scaled = e[0] * pt[0].z + e[1] * pt[1].z + e[2] * pt[2].z;

        // Now check if the sign of sum is different from the sign of tScaled, it it is, then no good:
        if (sum_e < 0. && (time_scaled >= 0. || time_scaled < max_time * sum_e))
            || (sum_e > 0. && (time_scaled <= 0. || time_scaled > max_time * sum_e))
        {
            return false;
        };

        let inv_sum_e = 1. / sum_e;
        // The time of the intersection (make sure it's positive):
        time_scaled * inv_sum_e > 0.
    }

    pub fn intersect(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        int_info: RayIntInfo,
        mesh: &Mesh,
    ) -> Option<Interaction> {
        let poss = self.get_poss(mesh);

        let pt = [poss[0] - ray.org, poss[1] - ray.org, poss[2] - ray.org];
        let pt = [
            pt[0].permute(int_info.perm),
            pt[1].permute(int_info.perm),
            pt[2].permute(int_info.perm),
        ];
        let pt = [
            Vec3 {
                x: int_info.shear.x * pt[0].z + pt[0].x,
                y: int_info.shear.y * pt[0].z + pt[0].y,
                z: pt[0].z,
            },
            Vec3 {
                x: int_info.shear.x * pt[1].z + pt[1].x,
                y: int_info.shear.y * pt[1].z + pt[1].y,
                z: pt[1].z,
            },
            Vec3 {
                x: int_info.shear.x * pt[2].z + pt[2].x,
                y: int_info.shear.y * pt[2].z + pt[2].y,
                z: pt[2].z,
            },
        ];

        // Calculate the edge function results:
        let e = [
            pt[1].x * pt[2].y - pt[1].y * pt[2].x,
            pt[2].x * pt[0].y - pt[2].y * pt[0].x,
            pt[0].x * pt[1].y - pt[0].y * pt[1].x,
        ];

        // Check if our ray lands outside of the edges of the triangle:
        if (e[0] < 0. || e[1] < 0. || e[2] < 0.) && (e[0] > 0. || e[1] > 0. || e[2] > 0.) {
            return None;
        };

        let sum_e = e[0] + e[1] + e[2];
        // Checks if it's a degenerate triangle:
        if sum_e == 0. {
            return None;
        };

        // Now we finish transforming the z value:
        let pt = [
            Vec3 {
                x: pt[0].x,
                y: pt[0].y,
                z: pt[0].z * int_info.shear.z,
            },
            Vec3 {
                x: pt[1].x,
                y: pt[1].y,
                z: pt[1].z * int_info.shear.z,
            },
            Vec3 {
                x: pt[2].x,
                y: pt[2].y,
                z: pt[2].z * int_info.shear.z,
            },
        ];

        let time_scaled = e[0] * pt[0].z + e[1] * pt[1].z + e[2] * pt[2].z;

        // Now check if the sign of sum is different from the sign of tScaled, it it is, then no good:
        if (sum_e < 0. && (time_scaled >= 0. || time_scaled < max_time * sum_e))
            || (sum_e > 0. && (time_scaled <= 0. || time_scaled > max_time * sum_e))
        {
            return None;
        };

        let inv_sum_e = 1. / sum_e;
        // The time of the intersection:
        let time = time_scaled * inv_sum_e;

        if time <= 0. {
            return None;
        }

        // Baycentric coordinates:
        let b = [e[0] * inv_sum_e, e[1] * inv_sum_e, e[2] * inv_sum_e];
        // The hit point:
        let p = poss[0].scale(b[0]) + poss[1].scale(b[1]) + poss[2].scale(b[2]);

        // The edges along the triangle:
        let dp02 = poss[0] - poss[2];
        let dp12 = poss[1] - poss[2];
        // The geometric normal of the hitpoint (according to just the position info)
        let n = dp02.cross(dp12).normalize();

        // Get the UV coordinates:
        let uvs = if mesh.has_uvs() {
            self.get_uvs(mesh)
        } else {
            [
                Vec2 { x: 0., y: 0. },
                Vec2 { x: 1., y: 0. },
                Vec2 { x: 1., y: 1. },
            ]
        };
        // Calculate the uv point where we intersect now:
        let uv = uvs[0].scale(b[0]) + uvs[1].scale(b[1]) + uvs[2].scale(b[2]);

        // Matrix entries for calculating dpdu and dpdv:
        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];
        let det = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let is_degen_uv = det.abs() < 1e-8; // This is quite a hack, so we should do something about this
        let inv_det = if is_degen_uv { 0. } else { 1. / det };

        // Compute triangle partial derivatives:
        // These vectors are parallel to the triangle:
        let (dpdu, dpdv) = if is_degen_uv {
            coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
        } else {
            // Solve the system:
            let dpdu = (dp02.scale(duv12[1]) - dp12.scale(duv02[1])).scale(inv_det);
            let dpdv = (dp02.scale(-duv12[0]) + dp12.scale(duv02[0])).scale(inv_det);
            if dpdu.cross(dpdv).length2() == 0. {
                coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
            } else {
                (dpdu, dpdv)
            }
        };

        // TODO: texture stuff goes here

        // Calculate the shading normals now:
        let shading_n = if mesh.has_nrm() {
            n // No normal information was provided, so we use the calculated normal.
        } else {
            let norms = self.get_nrms(mesh);
            let sn = norms[0].scale(b[0]) + norms[1].scale(b[1]) + norms[2].scale(b[2]);
            if sn.length2() == 0. {
                n
            } else {
                sn.normalize()
            }
        };
        // Update n with the new shading normal from the provided normal:
        let n = align(shading_n, n);

        // Calculate the shading dndu and dndv values:
        let (shading_dndu, shading_dndv) = if mesh.has_nrm() {
            (Vec3::zero(), Vec3::zero())
        } else {
            let norms = self.get_nrms(mesh);
            let dn02 = norms[0] - norms[2];
            let dn12 = norms[1] - norms[2];

            if is_degen_uv {
                let dn = (norms[2] - norms[0]).cross(norms[1] - norms[0]);
                if dn.length2() == 0. {
                    (Vec3::zero(), Vec3::zero())
                } else {
                    coord_system(dn)
                }
            } else {
                let dndu = (dn02.scale(duv12[1]) - dn12.scale(duv02[1])).scale(inv_det);
                let dndv = (dn02.scale(-duv12[0]) + dn12.scale(duv02[0])).scale(inv_det);
                (dndu, dndv)
            }
        };

        // Calculate the shading tangents:
        let shading_dpdu = if mesh.has_tan() {
            let tans = self.get_tans(mesh);
            let st = tans[0].scale(b[0]) + tans[1].scale(b[1]) + tans[2].scale(b[2]);
            if st.length2() == 0. {
                dpdu.normalize() // Just the same dpdu value as before
            } else {
                st.normalize()
            }
        } else {
            dpdu.normalize()
        };

        // Calculate the shaind bitangent:
        let (shading_dpdu, shading_dpdv) = {
            let sbt = shading_n.cross(shading_dpdu);
            if sbt.length2() > 0. {
                (sbt.cross(shading_dpdu), sbt.normalize())
            } else {
                coord_system(shading_n)
            }
        };

        let wo = -ray.dir;

        Some(Interaction {
            p,
            n,
            wo,
            time,
            uv,
            dpdu,
            dpdv,
            shading_n,
            shading_dpdu,
            shading_dpdv,
            shading_dndu,
            shading_dndv,
        })
    }

    pub fn bound(&self, mesh: &Mesh) -> BBox3<f64> {
        let poss = self.get_poss(mesh);
        BBox3::from_pnts(poss[0], poss[1]).combine_pnt(poss[2])
    }

    pub fn centroid(&self, mesh: &Mesh) -> Vec3<f64> {
        let poss = self.get_poss(mesh);
        (poss[0] + poss[1] + poss[2]).scale(1. / 3.)
    }

    pub fn area(&self, mesh: &Mesh) -> f64 {
        let poss = self.get_poss(mesh);
        (poss[0] - poss[1]).cross(poss[0] - poss[2]).length() * 0.5
    }

    // All of these are marked as unsafe because we always assume that the
    // mesh objects are created with correct triangle informations.
    // NOTE: only call these if you are certain that these values are present:

    // Returns the positions that make up a triangle:
    pub fn get_poss(&self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh.get_pos(self.indices[0]),
                mesh.get_pos(self.indices[1]),
                mesh.get_pos(self.indices[2]),
            ]
        }
    }

    // Returns the normals that make up a triangle:
    pub fn get_nrms(&self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh.get_nrm(self.indices[0]),
                mesh.get_nrm(self.indices[1]),
                mesh.get_nrm(self.indices[2]),
            ]
        }
    }

    // Returns the tangents that make up a triangle:
    pub fn get_tans(&self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh.get_tan(self.indices[0]),
                mesh.get_tan(self.indices[1]),
                mesh.get_tan(self.indices[2]),
            ]
        }
    }

    // Returns the uv's at that location:
    pub fn get_uvs(&self, mesh: &Mesh) -> [Vec2<f64>; 3] {
        unsafe {
            [
                mesh.get_uvs(self.indices[0]),
                mesh.get_uvs(self.indices[1]),
                mesh.get_uvs(self.indices[2]),
            ]
        }
    }
}

//
// The MeshBVH is a special structure for accelerating Mesh intersection.
//

pub struct MeshBVH {
    mesh: Mesh, // The mesh of the BVH (the BVH owns the mesh as it's specially modified)
    linear_nodes: Vec<LinearNode>, // The nodes that make up the tree
    bound: BBox3<f64>, // The overall bounding box of the entire BVH
}

impl MeshBVH {
    // Number of buckets used for SAH:
    const BUCKET_COUNT: usize = 12;
    const ALLOC_STACK_SIZE: usize = 1024 * 1024 / 32; // I might specify something else later

    // Constructs a BVH given a mesh and the max number of triangles per leaf node.
    // The BVH will become the owner of the mesh when doing this.
    pub fn new(mesh: Mesh, max_tri_per_node: u32) -> Self {
        // First we record any triangle information we may need:
        let tris_raw = mesh.get_tri_raw();
        let mut tris_info = Vec::with_capacity(tris_raw.len());
        for (i, tri) in tris_raw.iter().enumerate() {
            tris_info.push(TriangleInfo {
                tri_index: i as u32,
                centroid: tri.centroid(&mesh),
                bound: tri.bound(&mesh),
            });
        }

        // Now we can go ahead and construct the tree:
        Self::construct_tree(mesh, tris_info, max_tri_per_node)
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn geom_bound(&self) -> BBox3<f64> {
        self.bound
    }

    pub fn intersect_test(&self, ray: Ray<f64>, max_time: f64, int_info: RayIntInfo) -> bool {
        // This function has to be very efficient, so I'll be using a lot of unsafe code
        // here (but everything I'm doing should still be defined behavior).

        let inv_dir = ray.dir.inv_scale(1.);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut node_stack: [usize; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut node_stack_index = 0usize;
        let mut curr_node_index = 0usize;

        loop {
            let curr_node = *unsafe { self.linear_nodes.get_unchecked(curr_node_index) };
            if curr_node
                .bound
                .intersect_test(ray, max_time, inv_dir, is_dir_neg)
            {
                match curr_node.kind {
                    LinearNodeKind::Leaf {
                        tri_start_index,
                        tri_end_index,
                    } => {
                        let tri_start = tri_start_index as usize;
                        let tri_end = tri_end_index as usize;
                        unsafe {
                            for tri in self
                                .mesh
                                .get_tri_raw()
                                .get_unchecked(tri_start..tri_end)
                                .iter()
                            {
                                if tri.intersect_test(ray, max_time, int_info, &self.mesh) {
                                    return true;
                                }
                            }
                        }

                        // Pop the stack (if it's empty, we are done):
                        if node_stack_index == 0usize {
                            return false;
                        }
                        node_stack_index -= 1;
                        curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
                    }
                    LinearNodeKind::Interior {
                        right_child_index,
                        split_axis,
                    } => {
                        // Check which child it's most likely to be:
                        if is_dir_neg[split_axis as usize] {
                            // Push the first child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                curr_node_index + 1;
                            node_stack_index += 1;
                            curr_node_index = right_child_index as usize;
                        } else {
                            // Push the second child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                right_child_index as usize;
                            node_stack_index += 1;
                            curr_node_index += 1; // the first child
                        }
                    }
                }
            // If we don't hit it, then we try another item from the stack:
            } else {
                if node_stack_index == 0usize {
                    return false;
                }
                node_stack_index -= 1;
                curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
            }
        }
    }

    pub fn intersect(
        &self,
        ray: Ray<f64>,
        mut max_time: f64,
        int_info: RayIntInfo,
    ) -> Option<Interaction> {
        // This function has to be very efficient, so I'll be using a lot of unsafe code
        // here (but everything I'm doing should still be defined behavior).

        let inv_dir = ray.dir.inv_scale(1.);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut node_stack: [usize; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut node_stack_index = 0usize;
        let mut curr_node_index = 0usize;

        // This is the final result:
        let mut result = None;

        loop {
            let curr_node = *unsafe { self.linear_nodes.get_unchecked(curr_node_index) };
            if curr_node
                .bound
                .intersect_test(ray, max_time, inv_dir, is_dir_neg)
            {
                match curr_node.kind {
                    LinearNodeKind::Leaf {
                        tri_start_index,
                        tri_end_index,
                    } => {
                        let tri_start = tri_start_index as usize;
                        let tri_end = tri_end_index as usize;
                        unsafe {
                            for tri in self
                                .mesh
                                .get_tri_raw()
                                .get_unchecked(tri_start..tri_end)
                                .iter()
                            {
                                if let Some(intersection) =
                                    tri.intersect(ray, max_time, int_info, &self.mesh)
                                {
                                    // Update the max time for more efficient culling:
                                    max_time = intersection.time;
                                    // Can't return immediately, have to make sure this is the closest intersection
                                    result = Some(intersection);
                                }
                            }
                        }

                        // Pop the stack (if it's empty, we are done):
                        if node_stack_index == 0usize {
                            break;
                        }
                        node_stack_index -= 1;
                        curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
                    }
                    LinearNodeKind::Interior {
                        right_child_index,
                        split_axis,
                    } => {
                        // Check which child it's most likely to be:
                        if is_dir_neg[split_axis as usize] {
                            // Push the first child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                curr_node_index + 1;
                            node_stack_index += 1;
                            curr_node_index = right_child_index as usize;
                        } else {
                            // Push the second child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                right_child_index as usize;
                            node_stack_index += 1;
                            curr_node_index += 1; // the first child
                        }
                    }
                }
            // If we don't hit it, then we try another item from the stack:
            } else {
                if node_stack_index == 0usize {
                    break;
                }
                node_stack_index -= 1;
                curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
            }
        }

        result
    }

    // Given a mesh, triangle info (as passed by new), and the number of triangles per node,
    // construct a tree:
    fn construct_tree(
        mut mesh: Mesh,
        mut tris_info: Vec<TriangleInfo>,
        max_tri_per_node: u32,
    ) -> Self {
        // It would probably make more sense to create a better allocator for nodes then by doing
        // it this way, that way we could maintain pointers instead.
        let allocator = StackAlloc::new(Self::ALLOC_STACK_SIZE);
        // The new triangles that will replace the ones in Mesh (they will be ordered
        // in the correct manner):
        let mut new_tris = Vec::with_capacity(mesh.num_tris() as usize);

        // Construct the regular tree first (that isn't flat):
        let (root_node, bound) = Self::recursive_construct_tree(
            max_tri_per_node,
            &mesh,
            &mut tris_info,
            &mut new_tris,
            &allocator,
        );

        // Repalce the trianlges in the mesh with the reordered triangles:
        mesh.update_tris(new_tris);
        // Now we flatten the nodes for better memory and performance later down the line:
        let linear_nodes = Self::flatten_tree(allocator.get_alloc_count(), root_node);

        MeshBVH {
            mesh,
            linear_nodes,
            bound,
        }
    }

    // Need to specify the tree node and the total number of nodes.
    // Will return the linear nodes as a vector.
    fn flatten_tree(num_nodes: usize, root_node: &TreeNode) -> Vec<LinearNode> {
        // This will generate the linear nodes we care about:
        fn generate_linear_nodes(
            linear_nodes: &mut Vec<LinearNode>,
            curr_node: &TreeNode,
        ) -> usize {
            match *curr_node {
                TreeNode::Leaf {
                    bound,
                    tri_index,
                    num_tri,
                } => {
                    linear_nodes.push(LinearNode {
                        bound,
                        kind: LinearNodeKind::Leaf {
                            tri_start_index: tri_index,
                            tri_end_index: tri_index + num_tri,
                        },
                    });
                    linear_nodes.len() - 1
                }
                TreeNode::Interior {
                    bound,
                    children: (left, right),
                    split_axis,
                } => {
                    let curr_pos = linear_nodes.len();
                    // Temporarily "push" a value:
                    unsafe { linear_nodes.set_len(curr_pos + 1) };
                    generate_linear_nodes(linear_nodes, left);
                    let right_child_index = generate_linear_nodes(linear_nodes, right) as u32;
                    *unsafe { linear_nodes.get_unchecked_mut(curr_pos) } = LinearNode {
                        bound,
                        kind: LinearNodeKind::Interior {
                            right_child_index,
                            split_axis,
                        },
                    };
                    curr_pos
                }
            }
        }

        // First create a vector with the correct number of nodes:
        let mut linear_nodes = Vec::with_capacity(num_nodes);
        let cnt = generate_linear_nodes(&mut linear_nodes, root_node);
        linear_nodes
    }

    // Recursively constructs the tree.
    // Returns a reference to the root node of the tree and the bound of the entire tree:
    fn recursive_construct_tree<'a>(
        max_tri_per_node: u32,          // The maximum number of triangles per node.
        mesh: &Mesh,                    // The mesh we are currently constructing a BVH for.
        tri_infos: &mut [TriangleInfo], // The current slice of triangles we are working on.
        new_tris: &mut Vec<Triangle>,   // The correct order for the new triangles we care about.
        allocator: &'a StackAlloc<TreeNode<'a>>, // Allocator used to allocate the nodes. The lifetime of the nodes is that of the allocator
    ) -> (&'a TreeNode<'a>, BBox3<f64>) {
        // A bound over all of the triangles we are currently working with:
        let all_bound = tri_infos.iter().fold(BBox3::new(), |all_bound, tri_info| {
            all_bound.combine_bnd(tri_info.bound)
        });

        // If we only have one triangle, make a leaf:
        if tri_infos.len() == 1 {
            new_tris.push(mesh.get_tri(tri_infos[0].tri_index));
            return (
                allocator.push(TreeNode::Leaf {
                    bound: all_bound,
                    tri_index: (new_tris.len() - 1) as u32,
                    num_tri: 1,
                }),
                all_bound,
            );
        }

        // Otherwise, we want to split the tree into smaller parts:

        // The bound covering all of the centroids (used for SAH BVH construction):
        let centroid_bound = tri_infos
            .iter()
            .fold(BBox3::new(), |centroid_bound, tri_info| {
                centroid_bound.combine_pnt(tri_info.centroid)
            });

        // Now we want to split based on the largest dimension:
        let max_dim = centroid_bound.max_dim();

        // Check if the volume has volume 0, if so, then create a leaf node:
        if centroid_bound.pmax[max_dim] == centroid_bound.pmin[max_dim] {
            // Need to keep track of where we will be putting these triangles.
            let curr_tri_index = new_tris.len() as u32;
            for tri_info in tri_infos.iter() {
                new_tris.push(mesh.get_tri(tri_info.tri_index));
            }
            // Allocate the a new leaf node and push it:
            return (
                allocator.push(TreeNode::Leaf {
                    bound: all_bound,
                    tri_index: curr_tri_index,
                    num_tri: tri_infos.len() as u32,
                }),
                all_bound,
            );
        }

        // Figure out how to split the elements:
        // If we have less than 4 triangles, just split it evenly:
        let (tri_infos_left, tri_infos_right) = if tri_infos.len() <= 4 {
            // kth_by is essentially nth_element from C++.
            // Here, we reorder the triangles based on the value of the centroid
            // in the maximum dimension (dim).
            let mid = tri_infos.len() / 2;
            kth_by(tri_infos, mid, |tri_info0, tri_info1| {
                tri_info0.centroid[max_dim]
                    .partial_cmp(&tri_info1.centroid[max_dim])
                    .unwrap()
            });
            // Split the array:
            tri_infos.split_at_mut(mid)
        } else {
            // Otherwise, we perform this split based on surface-area heuristics:
            let mut buckets = [Bucket {
                count: 0,
                bound: BBox3::new(),
            }; Self::BUCKET_COUNT];

            for tri_info in tri_infos.iter() {
                // Get an index into where we are among the buckets:
                let bucket_ratio = centroid_bound.offset(tri_info.centroid)[max_dim];
                let bucket_index = if bucket_ratio == 1. {
                    Self::BUCKET_COUNT - 1
                } else {
                    ((Self::BUCKET_COUNT as f64) * bucket_ratio) as usize
                };

                let curr_bucket = &mut buckets[bucket_index];
                curr_bucket.count += 1;
                curr_bucket.bound = curr_bucket.bound.combine_bnd(tri_info.bound);
            }

            // Iterate over everything backwards, but ignore the first element to get the right
            // surface area values:
            let mut right_sa = [0f64; Self::BUCKET_COUNT - 1];
            let (_, right_count) = buckets[1..].iter().enumerate().rev().fold(
                (BBox3::new(), 0u32),
                |(right_bound, right_count), (i, bucket)| {
                    // Have to do this because enumerate starts at 0, always, not the index of the slice:
                    let right_bound = right_bound.combine_bnd(bucket.bound);
                    right_sa[i] = right_bound.surface_area();
                    (right_bound, right_count + bucket.count)
                },
            );

            // Now we can compute the values going forward to fill in the buckets.
            // We also must modify the right count as we decrement it over time:
            let mut costs = [0f64; Self::BUCKET_COUNT - 1];
            let total_sa = all_bound.surface_area();
            buckets[..(Self::BUCKET_COUNT - 1)].iter().enumerate().fold(
                (BBox3::new(), 0u32, right_count),
                |(left_bound, left_count, right_count), (i, bucket)| {
                    let left_bound = left_bound.combine_bnd(bucket.bound);
                    let left_count = left_count + bucket.count;
                    // Calculate the heuristic here:
                    costs[i] = 0.125
                        * ((left_count as f64) * left_bound.surface_area()
                            + (right_count as f64) * right_sa[i])
                        / total_sa;
                    (left_bound, left_count, right_count - buckets[i + 1].count)
                },
            );

            let (min_cost_index, &min_cost) = costs
                .iter() // returns a reference to the elements (so a &x essentially).
                .enumerate() // returns (i, &x), and max_by's lambda takes a reference. But coercion helps here:
                .min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
                .unwrap();

            // If this happens, then we should split more and continue our operations:
            if tri_infos.len() > (max_tri_per_node as usize) || min_cost < (tri_infos.len() as f64)
            {
                // Split (partition) based on bucket with min cost:
                partition(tri_infos, |tri_info| {
                    let bucket_ratio = centroid_bound.offset(tri_info.centroid)[max_dim];
                    let bucket_index = if bucket_ratio == 1. {
                        Self::BUCKET_COUNT - 1
                    } else {
                        ((Self::BUCKET_COUNT as f64) * bucket_ratio) as usize
                    };
                    bucket_index <= min_cost_index
                })
            } else {
                // Otherwise, it isn't worth it so continue the splitting process, so we
                // create a leaf here:
                let curr_tri_index = new_tris.len() as u32;
                for tri_info in tri_infos.iter() {
                    new_tris.push(mesh.get_tri(tri_info.tri_index));
                }
                return (
                    allocator.push(TreeNode::Leaf {
                        bound: all_bound,
                        tri_index: curr_tri_index,
                        num_tri: tri_infos.len() as u32,
                    }),
                    all_bound,
                );
            }
        };

        // Build the left and right nodes now:
        let (left_node, _) = Self::recursive_construct_tree(
            max_tri_per_node,
            mesh,
            tri_infos_left,
            new_tris,
            allocator,
        );
        let (right_node, _) = Self::recursive_construct_tree(
            max_tri_per_node,
            mesh,
            tri_infos_right,
            new_tris,
            allocator,
        );

        // Create a node and push it on:
        (
            allocator.push(TreeNode::Interior {
                bound: all_bound,
                children: (left_node, right_node),
                split_axis: max_dim as u8,
            }),
            all_bound,
        )
    }
}

// This is the bucket used for SAH splitting:
#[derive(Clone, Copy)]
struct Bucket {
    // Number of items in the current bucket:
    pub count: u32,
    // Bound for the current bucket:
    pub bound: BBox3<f64>,
}

// Structure used to construct the BVH:
#[derive(Clone, Copy)]
struct TriangleInfo {
    pub tri_index: u32,
    pub centroid: Vec3<f64>,
    pub bound: BBox3<f64>,
}

// This is the internal representation we have when initially building the tree.
// We later "flatten" the tree for efficient traversal.
#[derive(Clone, Copy)]
enum TreeNode<'a> {
    Leaf {
        bound: BBox3<f64>,
        tri_index: u32,
        num_tri: u32,
    },
    Interior {
        bound: BBox3<f64>,
        children: (&'a TreeNode<'a>, &'a TreeNode<'a>),
        split_axis: u8,
    },
}

//#[repr(align(32))] <- experimental, TODOL: add once not experimental
#[derive(Clone, Copy)]
enum LinearNodeKind {
    Leaf {
        tri_start_index: u32,
        tri_end_index: u32,
    },
    Interior {
        // left_child_index: it's always next to it in the array
        right_child_index: u32,
        split_axis: u8,
    },
}

#[derive(Clone, Copy)]
struct LinearNode {
    bound: BBox3<f64>,
    kind: LinearNodeKind,
}

//
// The MeshGeometry is the actual geometry that is present in the scene that is
// being rendered.
//

pub struct MeshGeometry<T: Transform> {
    mesh_bvh: MeshBVH,

    geom_to_world: T,

    // Because surface area calculations are costly,
    // we calculate them only once and cache the result here:
    surface_area: Cell<Option<f64>>,
}

impl<T: Transform> MeshGeometry<T> {
    pub fn new(mesh_bvh: MeshBVH, geom_to_world: T) -> Self {
        MeshGeometry {
            mesh_bvh,
            geom_to_world,
            surface_area: Cell::new(None),
        }
    }
}

impl<T: Transform> Geometry for MeshGeometry<T> {
    fn geom_bound(&self) -> BBox3<f64> {
        self.mesh_bvh.geom_bound()
    }

    fn world_bound(&self, t: f64) -> BBox3<f64> {
        self.geom_to_world.bound_motion(self.geom_bound(), t)
    }

    fn surface_area(&self) -> f64 {
        // Check if we already calculated this:
        if let Some(s) = self.surface_area.get() {
            return s;
        }

        // Otherwise, we go ahead and calculate the surface area
        // and cache it:
        let s = self.mesh_bvh.mesh().get_surface_area();
        self.surface_area.set(Some(s));
        s
    }

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> bool {
        // First we transform the ray into the appropraite space:
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        // Then we transform the ray itself and calculate the acceleration values:
        let ray = int_geom_to_world.inverse().ray(ray);
        let int_info = calc_rayintinfo(ray);
        self.mesh_bvh.intersect_test(ray, max_time, int_info)
    }

    fn intersect(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> Option<Interaction> {
        // First we transform the ray into the appropraite space:
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);
        // Then we transform the ray itself and calculate the acceleration values:
        let ray = int_geom_to_world.inverse().ray(ray);
        let int_info = calc_rayintinfo(ray);
        match self.mesh_bvh.intersect(ray, max_time, int_info) {
            // Don't forget to transform it back to the original space we care about:
            Some(i) => Some(int_geom_to_world.interaction(i)),
            _ => None,
        }
    }
}
