use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::util::{align, coord_system};
use crate::math::vector::{Vec2, Vec3, Vec3Perm};
use crate::bvh::{BVH, BVHObject};

use std::cell::Cell;

// TODO: figure out what needs to see what (triangle shouldn't be all public)

// MeshData represents the collection of data used to represent
// the 3D geometry.
#[derive(Clone, Debug)]
pub struct MeshData {
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
impl MeshData {
    // Important thing to note, if has_tan = true, then has_nrm must be true as well. If this is not
    // the case, bad things will happen....
    pub fn new(
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

        MeshData {
            data,
            has_nrm,
            has_tan,
            has_uvs,
            num_prop,
            num_vert,
        }
    }

    pub fn get_data_raw(&self) -> &Vec<f32> {
        &self.data
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

    pub fn get_surface_area(&self, triangles: &Vec<Triangle>) -> f64 {
        triangles.iter().fold(0., |area, tri| area + tri.get_area(self))
    }

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
        mesh_data: &MeshData,
    ) -> bool {
        let poss = self.get_poss(mesh_data);

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
        mesh_data: &MeshData,
    ) -> Option<Interaction> {
        let poss = self.get_poss(mesh_data);

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
        let uvs = if mesh_data.has_uvs() {
            self.get_uvs(mesh_data)
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
        let shading_n = if mesh_data.has_nrm() {
            n // No normal information was provided, so we use the calculated normal.
        } else {
            let norms = self.get_nrms(mesh_data);
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
        let (shading_dndu, shading_dndv) = if mesh_data.has_nrm() {
            (Vec3::zero(), Vec3::zero())
        } else {
            let norms = self.get_nrms(mesh_data);
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
        let shading_dpdu = if mesh_data.has_tan() {
            let tans = self.get_tans(mesh_data);
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

    pub fn get_bound(&self, mesh_data: &MeshData) -> BBox3<f64> {
        let poss = self.get_poss(mesh_data);
        BBox3::from_pnts(poss[0], poss[1]).combine_pnt(poss[2])
    }

    pub fn get_centroid(&self, mesh_data: &MeshData) -> Vec3<f64> {
        let poss = self.get_poss(mesh_data);
        (poss[0] + poss[1] + poss[2]).scale(1. / 3.)
    }

    pub fn get_area(&self, mesh_data: &MeshData) -> f64 {
        let poss = self.get_poss(mesh_data);
        (poss[0] - poss[1]).cross(poss[0] - poss[2]).length() * 0.5
    }

    // All of these are marked as unsafe because we always assume that the
    // mesh objects are created with correct triangle informations.
    // NOTE: only call these if you are certain that these values are present:

    // Returns the positions that make up a triangle:
    pub fn get_poss(&self, mesh_data: &MeshData) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh_data.get_pos(self.indices[0]),
                mesh_data.get_pos(self.indices[1]),
                mesh_data.get_pos(self.indices[2]),
            ]
        }
    }

    // Returns the normals that make up a triangle:
    pub fn get_nrms(&self, mesh_data: &MeshData) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh_data.get_nrm(self.indices[0]),
                mesh_data.get_nrm(self.indices[1]),
                mesh_data.get_nrm(self.indices[2]),
            ]
        }
    }

    // Returns the tangents that make up a triangle:
    pub fn get_tans(&self, mesh_data: &MeshData) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh_data.get_tan(self.indices[0]),
                mesh_data.get_tan(self.indices[1]),
                mesh_data.get_tan(self.indices[2]),
            ]
        }
    }

    // Returns the uv's at that location:
    pub fn get_uvs(&self, mesh_data: &MeshData) -> [Vec2<f64>; 3] {
        unsafe {
            [
                mesh_data.get_uvs(self.indices[0]),
                mesh_data.get_uvs(self.indices[1]),
                mesh_data.get_uvs(self.indices[2]),
            ]
        }
    }
}

impl BVHObject for Triangle {
    // Until Rust gets support for generic associated types that allow
    // for proper references with lifetimes, this will have to do:
    type IntParam = (RayIntInfo, *const MeshData);
    type DataParam = MeshData;

    // curr_time is not needed as BVHObject will work in geometry space only:
    fn intersect_test(&self, ray: Ray<f64>, max_time: f64, _: f64, &(ray_int_info, mesh_data): &Self::IntParam) -> bool {
        unsafe {
            // Dirty, I know:
            Triangle::intersect_test(self, ray, max_time, ray_int_info, &*mesh_data)
        }
    }

    // curr_time is not needed as BVHObject will work in geometry space only:
    fn intersect(
        &self,
        ray: Ray<f64>,
        max_time: f64,
        _: f64,
        &(ray_int_info, mesh_data): &Self::IntParam,
    ) -> Option<Interaction> {
        unsafe {
            // Dirty, I know:
            Triangle::intersect(self, ray, max_time, ray_int_info, &*mesh_data)
        }
    }

    fn get_centroid(&self, data: &Self::DataParam) -> Vec3<f64> {
        Triangle::get_centroid(self, data)
    }

    fn get_bound(&self, data: &Self::DataParam) -> BBox3<f64> {
        Triangle::get_bound(self, data)
    }
}

pub struct Mesh {
    mesh_data: MeshData,
    bvh: BVH<Triangle>,

    // Because surface area calculations are costly,
    // we calculate them only once and cache the result here:
    surface_area: Cell<Option<f64>>,
}

impl Mesh {
    // Might make this a user-defined setting in the future, not sure.
    const MAX_TRI_PER_NODE: usize = 16;

    pub fn new(mesh_data: MeshData, triangles: Vec<Triangle>) -> Self {
        Mesh {
            mesh_data,
            bvh: BVH::new(triangles, Self::MAX_TRI_PER_NODE, &mesh_data),
            surface_area: Cell::new(None),
        }
    }
}

impl Geometry for Mesh {
    fn get_bound(&self) -> BBox3<f64> {
        self.bvh.get_bound()
    }

    // TODO: calculate the centroid of a mesh:
    fn get_centroid(&self) -> Vec3<f64> {
        Vec3::zero()
    }

    fn get_surface_area(&self) -> f64 {
        // Check if we already calculated this:
        if let Some(s) = self.surface_area.get() {
            return s;
        }

        // Otherwise, we go ahead and calculate the surface area
        // and cache it:
        let s = self.mesh_data.get_surface_area(self.bvh.get_objects());
        self.surface_area.set(Some(s));
        s
    }

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64) -> bool {
        let ray_int_info = calc_rayintinfo(ray);
        // Because in geometry space we aren't moving, curr_time is not needed and we always set it to 0:
        self.bvh.intersect_test(ray, max_time, 0., &(ray_int_info, &self.mesh_data as *const MeshData))
    }

    fn intersect(&self, ray: Ray<f64>, max_time: f64) -> Option<Interaction> {
        let ray_int_info = calc_rayintinfo(ray);
        // Because in geometry space we aren't moving, curr_time is not needed and we always set it to 0:
        self.bvh.intersect(ray, max_time, 0., &(ray_int_info, &self.mesh_data as *const MeshData))
    }
}
