use crate::math::ray::Ray;
use crate::math::util::{align, coord_system, gamma_f32};
use crate::math::vector::{Vec2f, Vec3Perm, Vec3d, Vec3f};

// Compact mesh storage that should be cache friendly:
// (actually, the only reason I did it this way was because of the way mesh loading worked):
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
    ) -> Mesh {
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

    pub fn num_tri(&self) -> u32 {
        self.tris.len() as u32
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

    pub fn get_tri(&self, index: u32) -> Triangle {
        unsafe { *self.tris.get_unchecked(index as usize) }
    }

    pub fn get_pos(&self, index: u32) -> Vec3f {
        let index = (self.num_prop as usize) * (index as usize);
        unsafe {
            Vec3f {
                x: *self.data.get_unchecked(index + 0usize),
                y: *self.data.get_unchecked(index + 1usize),
                z: *self.data.get_unchecked(index + 2usize),
            }
        }
    }

    // make sure you have normals first...
    pub fn get_nrm(&self, index: u32) -> Vec3f {
        debug_assert!(self.has_nrm());

        // If we have normal information, it'll always follow the position:
        let index = (self.num_prop as usize) * (index as usize) + 3usize;
        unsafe {
            Vec3f {
                x: *self.data.get_unchecked(index + 0usize),
                y: *self.data.get_unchecked(index + 1usize),
                z: *self.data.get_unchecked(index + 2usize),
            }
        }
    }

    pub fn get_tan(&self, index: u32) -> Vec3f {
        debug_assert!(self.has_tan());

        // If we have tangent information, it will always follow normal position:
        let index = (self.num_prop as usize) * (index as usize) + 6usize;
        unsafe {
            Vec3f {
                x: *self.data.get_unchecked(index + 0usize),
                y: *self.data.get_unchecked(index + 1usize),
                z: *self.data.get_unchecked(index + 2usize),
            }
        }
    }

    pub fn get_uvs(&self, index: u32) -> Vec2f {
        debug_assert!(self.has_uvs());

        // Here we have to do a bit more work, because UVs cana technically belong anywhere:
        let index = (self.num_prop as usize) * (index as usize)
            + 3usize
            + ((self.has_nrm & 3u8) as usize)
            + ((self.has_tan & 3u8) as usize);
        unsafe {
            Vec2f {
                x: *self.data.get_unchecked(index + 0usize),
                y: *self.data.get_unchecked(index + 1usize),
            }
        }
    }

    pub fn set_pos(&mut self, index: u32, vec: Vec3f) {
        let index = (self.num_prop as usize) * (index as usize);
        unsafe {
            *self.data.get_unchecked_mut(index + 0usize) = vec.x;
            *self.data.get_unchecked_mut(index + 1usize) = vec.y;
            *self.data.get_unchecked_mut(index + 2usize) = vec.z;
        }
    }

    pub fn set_nrm(&mut self, index: u32, vec: Vec3f) {
        debug_assert!(self.has_nrm());

        let index = (self.num_prop as usize) * (index as usize) + 3usize;
        unsafe {
            *self.data.get_unchecked_mut(index + 0usize) = vec.x;
            *self.data.get_unchecked_mut(index + 1usize) = vec.y;
            *self.data.get_unchecked_mut(index + 2usize) = vec.z;
        }
    }

    pub fn set_tan(&mut self, index: u32, vec: Vec3f) {
        debug_assert!(self.has_tan());

        let index = (self.num_prop as usize) * (index as usize) + 6usize;
        unsafe {
            *self.data.get_unchecked_mut(index + 0usize) = vec.x;
            *self.data.get_unchecked_mut(index + 1usize) = vec.y;
            *self.data.get_unchecked_mut(index + 2usize) = vec.z;
        }
    }

    pub fn set_uvs(&mut self, index: u32, vec: Vec2f) {
        debug_assert!(self.has_uvs());

        let index = (self.num_prop as usize) * (index as usize)
            + 3usize
            + ((self.has_nrm & 3u8) as usize)
            + ((self.has_tan & 3u8) as usize);
        unsafe {
            *self.data.get_unchecked_mut(index + 0usize) = vec.x;
            *self.data.get_unchecked_mut(index + 1usize) = vec.y;
        }
    }
}

// A struct that stores information about the intersection
// of a mesh:
pub struct Intersection {
    pub p: Vec3f,     // intersection point
    pub n: Vec3f,     // geometric normal (of triangle)
    pub wo: Vec3f,    // direction of intersection leaving the point
    pub p_err: Vec3f, // error at the intersection point

    pub time: f32, // time when the intersection occured

    pub uv: Vec2f,   // uv coordinate at the intersection
    pub dpdu: Vec3f, // vectors parallel to the triangle
    pub dpdv: Vec3f,

    pub shading_n: Vec3f,    // the shading normal at this point
    pub shading_dpdu: Vec3f, // the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3f,
    pub shading_dndu: Vec3f, // the shading dndu, dndv at this point
    pub shading_dndv: Vec3f,
}

// Stores extra information used to speed up ray intersection calculations:
pub struct RayIntInfo {
    shear: Vec3f,
    perm_dir: Vec3f,
    perm: Vec3Perm,
}

// Given a ray, calculates the ray intersection information used for
// efficient ray-triangle intersection.
pub fn calc_rayintinfo(ray: &Ray) -> RayIntInfo {
    let z = ray.dir.abs().max_dim();
    let x = if z == 2usize { 0usize } else { z + 1usize };
    let y = if x == 2usize { 0usize } else { x + 1usize };

    let perm = Vec3Perm::new(x, y, z);
    let perm_dir = ray.dir.permute(perm);

    let inv_perm_dir_z = 1f32 / perm_dir.z;
    let shear = Vec3f {
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
    pub fn intersect_test(&self, ray: &Ray, int_info: &RayIntInfo, mesh: &Mesh) -> bool {
        let poss = self.get_poss(mesh);

        // // NOTE: if you decide to include this, dp is used somewhere else in the code
        // // Check for bad triangles:
        // let dp = (poss[2] - poss[0]).cross(poss[1] - poss[0]);
        // if (dp.length2() == 0f32) {
        //     return false;
        // }

        let pt = [poss[0] - ray.org, poss[1] - ray.org, poss[2] - ray.org];
        let pt = [
            pt[0].permute(int_info.perm),
            pt[1].permute(int_info.perm),
            pt[2].permute(int_info.perm),
        ];
        let pt = [
            Vec3f {
                x: int_info.shear.x * pt[0].z + pt[0].x,
                y: int_info.shear.y * pt[0].z + pt[0].y,
                z: pt[0].z,
            },
            Vec3f {
                x: int_info.shear.x * pt[1].z + pt[1].x,
                y: int_info.shear.y * pt[1].z + pt[1].y,
                z: pt[1].z,
            },
            Vec3f {
                x: int_info.shear.x * pt[2].z + pt[2].x,
                y: int_info.shear.y * pt[2].z + pt[2].y,
                z: pt[2].z,
            },
        ];

        // Calculate the edge function results:
        let e = {
            let e0 = pt[1].x * pt[2].y - pt[1].y * pt[2].x;
            let e1 = pt[2].x * pt[0].y - pt[2].y * pt[0].x;
            let e2 = pt[0].x * pt[1].y - pt[0].y * pt[1].x;

            // Check if we need to recalculate the result in higher precision:
            if e0 == 0f32 || e1 == 0f32 || e2 == 0f32 {
                let dpt0 = Vec3d {
                    x: pt[0].x as f64,
                    y: pt[0].y as f64,
                    z: pt[0].z as f64,
                };
                let dpt1 = Vec3d {
                    x: pt[1].x as f64,
                    y: pt[1].y as f64,
                    z: pt[1].z as f64,
                };
                let dpt2 = Vec3d {
                    x: pt[2].x as f64,
                    y: pt[2].y as f64,
                    z: pt[2].z as f64,
                };

                let de0 = dpt1.x * dpt2.y - dpt1.y * dpt2.x;
                let de1 = dpt2.x * dpt0.y - dpt2.y * dpt0.x;
                let de2 = dpt0.x * dpt1.y - dpt0.y * dpt1.x;

                [de0 as f32, de1 as f32, de2 as f32]
            } else {
                [e0, e1, e2]
            }
        };

        // Check if our ray lands outside of the edges of the triangle:
        if (e[0] < 0f32 || e[1] < 0f32 || e[2] < 0f32)
            && (e[0] > 0f32 || e[1] > 0f32 || e[2] > 0f32)
        {
            return false;
        };

        let sum_e = e[0] + e[1] + e[2];
        // Checks if it's a degenerate triangle:
        if sum_e == 0f32 {
            return false;
        };

        // Now we finish transforming the z value:
        let pt = [
            Vec3f {
                x: pt[0].x,
                y: pt[0].y,
                z: pt[0].z * int_info.shear.z,
            },
            Vec3f {
                x: pt[1].x,
                y: pt[1].y,
                z: pt[1].z * int_info.shear.z,
            },
            Vec3f {
                x: pt[2].x,
                y: pt[2].y,
                z: pt[2].z * int_info.shear.z,
            },
        ];

        let time_scaled = e[0] * pt[0].z + e[1] * pt[1].z + e[2] * pt[2].z;

        // Now check if the sign of sum is different from the sign of tScaled, it it is, then no good:
        if (sum_e < 0f32 && (time_scaled >= 0f32 || time_scaled < ray.max_time * sum_e))
            || (sum_e > 0f32 && (time_scaled <= 0f32 || time_scaled > ray.max_time * sum_e))
        {
            return false;
        };

        let inv_sum_e = 1f32 / sum_e;
        // The time of the intersection:
        let time = time_scaled * inv_sum_e;

        // Perform some error detection:
        let abs_pt = [pt[0].abs(), pt[1].abs(), pt[2].abs()];
        let abs_e = [e[0].abs(), e[1].abs(), e[2].abs()];

        let max_z = abs_pt[0].z.max(abs_pt[1].z.max(abs_pt[2].z));
        let delta_z = gamma_f32(3) * max_z;

        let max_x = abs_pt[0].x.max(abs_pt[1].x.max(abs_pt[2].x));
        let delta_x = gamma_f32(5) * (max_x + max_z);

        let max_y = abs_pt[0].y.max(abs_pt[1].y.max(abs_pt[2].y));
        let delta_y = gamma_f32(5) * (max_y + max_z);

        let delta_e = 2f32 * (gamma_f32(2) * max_x * max_y + delta_y * max_x + delta_x * max_y);

        let max_e = abs_e[0].max(abs_e[1].max(abs_e[2]));
        let delta_t = 3f32
            * (gamma_f32(3) * max_e * max_z + delta_e * max_z + delta_z * max_e)
            * inv_sum_e.abs();

        time > delta_t
    }

    pub fn intersect(&self, ray: &Ray, int_info: &RayIntInfo, mesh: &Mesh) -> Option<Intersection> {
        let poss = self.get_poss(mesh);

        // // NOTE: if you decide to include this, dp is used somewhere else in the code
        // // Check for bad triangles:
        // let dp = (poss[2] - poss[0]).cross(poss[1] - poss[0]);
        // if (dp.length2() == 0f32) {
        //     return false;
        // }

        let pt = [poss[0] - ray.org, poss[1] - ray.org, poss[2] - ray.org];
        let pt = [
            pt[0].permute(int_info.perm),
            pt[1].permute(int_info.perm),
            pt[2].permute(int_info.perm),
        ];
        let pt = [
            Vec3f {
                x: int_info.shear.x * pt[0].z + pt[0].x,
                y: int_info.shear.y * pt[0].z + pt[0].y,
                z: pt[0].z,
            },
            Vec3f {
                x: int_info.shear.x * pt[1].z + pt[1].x,
                y: int_info.shear.y * pt[1].z + pt[1].y,
                z: pt[1].z,
            },
            Vec3f {
                x: int_info.shear.x * pt[2].z + pt[2].x,
                y: int_info.shear.y * pt[2].z + pt[2].y,
                z: pt[2].z,
            },
        ];

        // Calculate the edge function results:
        let e = {
            let e0 = pt[1].x * pt[2].y - pt[1].y * pt[2].x;
            let e1 = pt[2].x * pt[0].y - pt[2].y * pt[0].x;
            let e2 = pt[0].x * pt[1].y - pt[0].y * pt[1].x;

            // Check if we need to recalculate the result in higher precision:
            if e0 == 0f32 || e1 == 0f32 || e2 == 0f32 {
                let dpt0 = Vec3d {
                    x: pt[0].x as f64,
                    y: pt[0].y as f64,
                    z: pt[0].z as f64,
                };
                let dpt1 = Vec3d {
                    x: pt[1].x as f64,
                    y: pt[1].y as f64,
                    z: pt[1].z as f64,
                };
                let dpt2 = Vec3d {
                    x: pt[2].x as f64,
                    y: pt[2].y as f64,
                    z: pt[2].z as f64,
                };

                let de0 = dpt1.x * dpt2.y - dpt1.y * dpt2.x;
                let de1 = dpt2.x * dpt0.y - dpt2.y * dpt0.x;
                let de2 = dpt0.x * dpt1.y - dpt0.y * dpt1.x;

                [de0 as f32, de1 as f32, de2 as f32]
            } else {
                [e0, e1, e2]
            }
        };

        // Check if our ray lands outside of the edges of the triangle:
        if (e[0] < 0f32 || e[1] < 0f32 || e[2] < 0f32)
            && (e[0] > 0f32 || e[1] > 0f32 || e[2] > 0f32)
        {
            return None;
        };

        let sum_e = e[0] + e[1] + e[2];
        // Checks if it's a degenerate triangle:
        if sum_e == 0f32 {
            return None;
        };

        // Now we finish transforming the z value:
        let pt = [
            Vec3f {
                x: pt[0].x,
                y: pt[0].y,
                z: pt[0].z * int_info.shear.z,
            },
            Vec3f {
                x: pt[1].x,
                y: pt[1].y,
                z: pt[1].z * int_info.shear.z,
            },
            Vec3f {
                x: pt[2].x,
                y: pt[2].y,
                z: pt[2].z * int_info.shear.z,
            },
        ];

        let time_scaled = e[0] * pt[0].z + e[1] * pt[1].z + e[2] * pt[2].z;

        // Now check if the sign of sum is different from the sign of tScaled, it it is, then no good:
        if (sum_e < 0f32 && (time_scaled >= 0f32 || time_scaled < ray.max_time * sum_e))
            || (sum_e > 0f32 && (time_scaled <= 0f32 || time_scaled > ray.max_time * sum_e))
        {
            return None;
        };

        let inv_sum_e = 1f32 / sum_e;
        // The time of the intersection:
        let time = time_scaled * inv_sum_e;

        // Perform some error detection:
        {
            let abs_pt = [pt[0].abs(), pt[1].abs(), pt[2].abs()];
            let abs_e = [e[0].abs(), e[1].abs(), e[2].abs()];

            let max_z = abs_pt[0].z.max(abs_pt[1].z.max(abs_pt[2].z));
            let delta_z = gamma_f32(3) * max_z;

            let max_x = abs_pt[0].x.max(abs_pt[1].x.max(abs_pt[2].x));
            let delta_x = gamma_f32(5) * (max_x + max_z);

            let max_y = abs_pt[0].y.max(abs_pt[1].y.max(abs_pt[2].y));
            let delta_y = gamma_f32(5) * (max_y + max_z);

            let delta_e = 2f32 * (gamma_f32(2) * max_x * max_y + delta_y * max_x + delta_x * max_y);

            let max_e = abs_e[0].max(abs_e[1].max(abs_e[2]));
            let delta_t = 3f32
                * (gamma_f32(3) * max_e * max_z + delta_e * max_z + delta_z * max_e)
                * inv_sum_e.abs();

            if time <= delta_t {
                return None;
            }
        }

        // Baycentric coordinates:
        let b = [e[0] * inv_sum_e, e[1] * inv_sum_e, e[2] * inv_sum_e];
        // The hit point:
        let p = poss[0].scale(b[0]) + poss[1].scale(b[1]) + poss[2].scale(b[2]);

        // Calculate the error at this point now:
        let p_err = {
            let x = (b[0] * poss[0].x).abs() + (b[1] * poss[1].x).abs() + (b[2] * poss[2].x).abs();
            let y = (b[0] * poss[0].y).abs() + (b[1] * poss[1].y).abs() + (b[2] * poss[2].y).abs();
            let z = (b[0] * poss[0].z).abs() + (b[1] * poss[1].z).abs() + (b[2] * poss[2].z).abs();
            (Vec3f { x, y, z }).scale(gamma_f32(7))
        };

        // The edges along the triangle:
        let dp02 = poss[0] - poss[2];
        let dp12 = poss[1] - poss[2];
        // The geometric normal of the hitpoint (according to just the position info)
        let n = dp02.cross(dp12).normalize();

        // Get the UV coordinates:
        let uvs = self.get_uvs(mesh);
        // Calculate the uv point where we intersect now:
        let uv = uvs[0].scale(b[0]) + uvs[1].scale(b[1]) + uvs[2].scale(b[2]);

        // Matrix entries for calculating dpdu and dpdv:
        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];
        let det = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let is_degen_uv = det.abs() < 1e-8f32; // This is quite a hack, so we should do something about this
        let inv_det = if is_degen_uv { 0f32 } else { 1f32 / det };

        // Compute triangle partial derivatives:
        // These vectors are parallel to the triangle:
        let (dpdu, dpdv) = if is_degen_uv {
            coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
        } else {
            // Solve the system:
            let dpdu = (dp02.scale(duv12[1]) - dp12.scale(duv02[1])).scale(inv_det);
            let dpdv = (dp02.scale(-duv12[0]) + dp12.scale(duv02[0])).scale(inv_det);
            if dpdu.cross(dpdv).length2() == 0f32 {
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
            if sn.length2() == 0f32 {
                n
            } else {
                sn.normalize()
            }
        };
        // Update n with the new shading normal from the provided normal:
        let n = align(shading_n, n);

        // Calculate the shading dndu and dndv values:
        let (shading_dndu, shading_dndv) = if mesh.has_nrm() {
            (Vec3f::zero(), Vec3f::zero())
        } else {
            let norms = self.get_nrms(mesh);
            let dn02 = norms[0] - norms[2];
            let dn12 = norms[1] - norms[2];

            if is_degen_uv {
                let dn = (norms[2] - norms[0]).cross(norms[1] - norms[0]);
                if dn.length2() == 0f32 {
                    (Vec3f::zero(), Vec3f::zero())
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
            if st.length2() == 0f32 {
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
            if sbt.length2() > 0f32 {
                (sbt.cross(shading_dpdu), sbt.normalize())
            } else {
                coord_system(shading_n)
            }
        };

        let wo = -ray.dir;

        Some(Intersection {
            p,
            n,
            wo,
            p_err,
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

    pub fn centroid(&self, mesh: &Mesh) -> Vec3f {
        let poss = self.get_poss(mesh);
        (poss[0] + poss[1] + poss[2]).scale(1f32 / 3f32)
    }

    // Might make unsafe if it improves performance:

    fn get_poss(&self, mesh: &Mesh) -> [Vec3f; 3] {
        unsafe {
            [
                mesh.get_pos(self.indices[0]),
                mesh.get_pos(self.indices[1]),
                mesh.get_pos(self.indices[2]),
            ]
        }
    }

    fn get_nrms(&self, mesh: &Mesh) -> [Vec3f; 3] {
        [
            mesh.get_nrm(self.indices[0]),
            mesh.get_nrm(self.indices[1]),
            mesh.get_nrm(self.indices[2]),
        ]
    }

    fn get_tans(&self, mesh: &Mesh) -> [Vec3f; 3] {
        [
            mesh.get_tan(self.indices[0]),
            mesh.get_tan(self.indices[1]),
            mesh.get_tan(self.indices[2]),
        ]
    }

    fn get_uvs(&self, mesh: &Mesh) -> [Vec2f; 3] {
        if mesh.has_uvs() {
            [
                mesh.get_uvs(self.indices[0]),
                mesh.get_uvs(self.indices[1]),
                mesh.get_uvs(self.indices[2]),
            ]
        } else {
            [
                Vec2f { x: 0f32, y: 0f32 },
                Vec2f { x: 1f32, y: 0f32 },
                Vec2f { x: 1f32, y: 1f32 },
            ]
        }
    }
}
