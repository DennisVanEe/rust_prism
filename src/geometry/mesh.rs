use crate::math::ray::Ray;
use crate::math::util::{align, coord_system};
use crate::math::vector::{Vec2f, Vec3f, Vec3d};

pub struct Mesh {
    pub tris: Vec<Triangle>,
    pub poss: Vec<Vec3f>,
    pub norms: Vec<Vec3f>,
    pub tans: Vec<Vec3f>,
    pub uvs: Vec<Vec2f>,
}

// A struct that stores information about the intersection
// of a mesh:
pub struct Intersection {
    pub p: Vec3f,
    pub n: Vec3f,
    pub wo: Vec3f,

    pub time: f32,

    pub uv: Vec2f,
    pub dpdu: Vec3f,
    pub dpdv: Vec3f,

    pub shading_n: Vec3f,
    pub shading_dpdu: Vec3f,
    pub shading_dpdv: Vec3f,
    pub shading_dndu: Vec3f,
    pub shading_dndv: Vec3f,
}

// Stores extra information used to speed up ray intersection calculations:
pub struct RayIntInfo {
    shear: Vec3f,
    perm_dir: Vec3f,
    perm: u32,
}

// Given a ray, calculates the ray intersection information used for
// efficient ray-triangle intersection.
pub fn calc_rayintinfo(ray: &Ray) -> RayIntInfo {
    let z = ray.dir.abs().max_dim() as u32;
    let x = if z == 2u32 { 0u32 } else { z + 1u32 };
    let y = if x == 2u32 { 0u32 } else { x + 1u32 };

    let perm = x + 2 * y + 4 * z;
    let perm_dir = ray.dir.permute(perm);

    let shear = Vec3f {
        x: -perm_dir.x / perm_dir.z,
        y: -perm_dir.y / perm_dir.z,
        z: 1f32 / perm_dir.z,
    };

    RayIntInfo {
        shear,
        perm_dir,
        perm,
    }
}

// The triangle itself doesn't store the mesh, it just stores the indices:
pub struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    pub fn intersect_test(&self, ray: &Ray, intinfo: &RayIntInfo, mesh: &Mesh) -> bool {
        let poss = self.get_poss(mesh);
        let pt = [poss[0] - ray.org, poss[1] - ray.org, poss[2] - ray.org];
        let pt = [
            pt[0].permute(intinfo.perm),
            pt[1].permute(intinfo.perm),
            pt[2].permute(intinfo.perm),
        ];
        let pt = [
            Vec3f {
                x: intinfo.shear.x * pt[0].z + pt[0].x,
                y: intinfo.shear.y * pt[0].z + pt[0].y,
                z: pt[0].z,
            },
            Vec3f {
                x: intinfo.shear.x * pt[1].z + pt[1].x,
                y: intinfo.shear.y * pt[1].z + pt[1].y,
                z: pt[1].z,
            },
            Vec3f {
                x: intinfo.shear.x * pt[2].z + pt[2].x,
                y: intinfo.shear.y * pt[2].z + pt[2].y,
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
                z: pt[0].z * intinfo.shear.z,
            },
            Vec3f {
                x: pt[1].x,
                y: pt[1].y,
                z: pt[1].z * intinfo.shear.z,
            },
            Vec3f {
                x: pt[2].x,
                y: pt[2].y,
                z: pt[2].z * intinfo.shear.z,
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
        // Finally, we check if the time is in the correct place:
        time > 1e-8f32
    }

    pub fn intersect(&self, ray: &Ray, intinfo: &RayIntInfo, mesh: &Mesh) -> Option<Intersection> {
        let poss = self.get_poss(mesh);
        let pt = [poss[0] - ray.org, poss[1] - ray.org, poss[2] - ray.org];
        let pt = [
            pt[0].permute(intinfo.perm),
            pt[1].permute(intinfo.perm),
            pt[2].permute(intinfo.perm),
        ];
        let pt = [
            Vec3f {
                x: intinfo.shear.x * pt[0].z + pt[0].x,
                y: intinfo.shear.y * pt[0].z + pt[0].y,
                z: pt[0].z,
            },
            Vec3f {
                x: intinfo.shear.x * pt[1].z + pt[1].x,
                y: intinfo.shear.y * pt[1].z + pt[1].y,
                z: pt[1].z,
            },
            Vec3f {
                x: intinfo.shear.x * pt[2].z + pt[2].x,
                y: intinfo.shear.y * pt[2].z + pt[2].y,
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
                z: pt[0].z * intinfo.shear.z,
            },
            Vec3f {
                x: pt[1].x,
                y: pt[1].y,
                z: pt[1].z * intinfo.shear.z,
            },
            Vec3f {
                x: pt[2].x,
                y: pt[2].y,
                z: pt[2].z * intinfo.shear.z,
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

        // Maybe pick something better here to determine if ray is too close:
        if time <= 1e-8f32 {
            return None;
        };

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
        let uvs = self.get_uvs(mesh);
        // Calculate the uv point where we intersect now:
        let uv = uvs[0].scale(b[0]) + uvs[1].scale(b[1]) + uvs[2].scale(b[2]);

        // Matrix entries for calculating dpdu and dpdv:
        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];
        let det = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let is_degen_uv = det.abs() < 1e-8f32;
        let inv_det = if is_degen_uv { 0f32 } else { 1f32 / det };

        // Compute triangle partial derivatives:
        // These vectors are parallel to the triangle:
        let (dpdu, dpdv) = if is_degen_uv {
            let ng = (poss[2] - poss[0]).cross(poss[1] - poss[0]);
            coord_system(ng)
        } else {
            // Solve the system:
            let dpdu = (dp02.scale(duv12[1]) - dp12.scale(duv02[1])).scale(inv_det);
            let dpdv = (dp02.scale(-duv12[0]) + dp12.scale(duv02[0])).scale(inv_det);
            if dpdu.cross(dpdv).length2() == 0f32 {
                let ng = (poss[2] - poss[0]).cross(poss[1] - poss[0]);
                coord_system(ng)
            } else {
                (dpdu, dpdv)
            }
        };

        // Calculate the shading normals now:
        let shading_n = if mesh.norms.is_empty() {
            n // No normal information was provided, so we use the calculated normal.
        } else {
            let norms = self.get_norms(mesh);
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
        let (shading_dndu, shading_dndv) = if mesh.norms.is_empty() {
            (
                Vec3f {
                    x: 0f32,
                    y: 0f32,
                    z: 0f32,
                },
                Vec3f {
                    x: 0f32,
                    y: 0f32,
                    z: 0f32,
                },
            )
        } else {
            let norms = self.get_norms(mesh);
            let dn02 = norms[0] - norms[2];
            let dn12 = norms[1] - norms[2];

            if is_degen_uv {
                let dn = (norms[2] - norms[0]).cross(norms[1] - norms[0]);
                if dn.length2() == 0f32 {
                    (
                        Vec3f {
                            x: 0f32,
                            y: 0f32,
                            z: 0f32,
                        },
                        Vec3f {
                            x: 0f32,
                            y: 0f32,
                            z: 0f32,
                        },
                    )
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
        let shading_dpdu = if mesh.tans.is_empty() {
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
            if sbt.length2() != 0f32 {
                (sbt.cross(shading_dpdu), sbt.normalize())
            } else {
                coord_system(shading_n)
            }
        };

        let wo = Vec3f {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        };

        Some(Intersection {
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

    pub fn centroid(&self, mesh: &Mesh) -> Vec3f {
        let poss = unsafe { self.get_poss(mesh) };
        (poss[0] + poss[1] + poss[2]).scale(1f32 / 3f32)
    }

    // Might make unsafe if it improves performance:

    unsafe fn get_poss(&self, mesh: &Mesh) -> [Vec3f; 3] {
        [
            *mesh.poss.get_unchecked(self.indices[0] as usize),
            *mesh.poss.get_unchecked(self.indices[1] as usize),
            *mesh.poss.get_unchecked(self.indices[2] as usize),
        ]
    }

    unsafe fn get_norms(&self, mesh: &Mesh) -> [Vec3f; 3] {
        [
            *mesh.norms.get_unchecked(self.indices[0] as usize),
            *mesh.norms.get_unchecked(self.indices[1] as usize),
            *mesh.norms.get_unchecked(self.indices[2] as usize),
        ]
    }

    unsafe fn get_tans(&self, mesh: &Mesh) -> [Vec3f; 3] {
        [
            *mesh.tans.get_unchecked(self.indices[0] as usize),
            *mesh.tans.get_unchecked(self.indices[1] as usize),
            *mesh.tans.get_unchecked(self.indices[2] as usize),
        ]
    }

    unsafe fn get_uvs(&self, mesh: &Mesh) -> [Vec2f; 3] {
        if mesh.uvs.is_empty() {
            [
                Vec2f { x: 0f32, y: 0f32 },
                Vec2f { x: 1f32, y: 0f32 },
                Vec2f { x: 1f32, y: 1f32 },
            ]
        } else {
            [
                *mesh.uvs.get_unchecked(self.indices[0] as usize),
                *mesh.uvs.get_unchecked(self.indices[1] as usize),
                *mesh.uvs.get_unchecked(self.indices[2] as usize),
            ]
        }
    }
}