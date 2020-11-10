use crate::bvh::{BVHObject, BVH};
use crate::geometry::Geometry;
use crate::interaction::{GeomSurf, SurfType, Surface};
use pmath;
use pmath::bbox::BBox3;
use pmath::ray::Ray;
use pmath::vector::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
struct RayIntInfo {
    shear: Vec3<f64>,
    perm_dir: Vec3<f64>,
    perm: Vec3<usize>,
}

impl RayIntInfo {
    fn new(ray: Ray<f64>) -> Self {
        let z = ray.dir.abs().max_dim();
        let x = if z == 2 { 0 } else { z + 1 };
        let y = if x == 2 { 0 } else { x + 1 };

        let perm = Vec3 { x, y, z };
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
}

#[derive(Clone, Copy, Debug)]
struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    fn area(self, mesh: &MeshData) -> f64 {
        let pos = self.pos(mesh);
        let a = pos[1] - pos[0];
        let b = pos[2] - pos[0];
        a.cross(b).length() * 0.5
    }

    fn pos(self, mesh: &MeshData) -> [Vec3<f64>; 3] {
        [
            mesh.pos[self.indices[0] as usize].to_f64(),
            mesh.pos[self.indices[1] as usize].to_f64(),
            mesh.pos[self.indices[2] as usize].to_f64(),
        ]
    }

    fn nrm(self, mesh: &MeshData) -> [Vec3<f64>; 3] {
        [
            mesh.nrm[self.indices[0] as usize].to_f64(),
            mesh.nrm[self.indices[1] as usize].to_f64(),
            mesh.nrm[self.indices[2] as usize].to_f64(),
        ]
    }

    fn tan(self, mesh: &MeshData) -> [Vec3<f64>; 3] {
        [
            mesh.tan[self.indices[0] as usize].to_f64(),
            mesh.tan[self.indices[1] as usize].to_f64(),
            mesh.tan[self.indices[2] as usize].to_f64(),
        ]
    }

    fn uvs(self, mesh: &MeshData) -> [Vec2<f64>; 3] {
        [
            mesh.uvs[self.indices[0] as usize].to_f64(),
            mesh.uvs[self.indices[1] as usize].to_f64(),
            mesh.uvs[self.indices[2] as usize].to_f64(),
        ]
    }
}

impl BVHObject for Triangle {
    type UserData = MeshData;

    /// Performs an intersection test for the specific triangle.
    fn intersect_test(&self, ray: Ray<f64>, mesh: &MeshData) -> bool {
        let int_info = RayIntInfo::new(ray);
        let poss = self.pos(mesh);

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

        let t_scaled = e[0] * pt[0].z + e[1] * pt[1].z + e[2] * pt[2].z;

        // Now check if the sign of sum is different from the sign of tScaled, it it is, then no good:
        if (sum_e < 0. && (t_scaled >= 0. || t_scaled < ray.t_far * sum_e))
            || (sum_e > 0. && (t_scaled <= 0. || t_scaled > ray.t_far * sum_e))
        {
            return false;
        };

        let inv_sum_e = 1. / sum_e;
        // The t of the intersection (make sure it's positive):
        t_scaled * inv_sum_e > 0.
    }

    fn intersect(&self, ray: Ray<f64>, mesh: &MeshData) -> Option<Surface> {
        let int_info = RayIntInfo::new(ray);
        let poss = self.pos(mesh);

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

        let t_scaled = e[0] * pt[0].z + e[1] * pt[1].z + e[2] * pt[2].z;

        // Now check if the sign of sum is different from the sign of tScaled, it it is, then no good:
        if (sum_e < 0. && (t_scaled >= 0. || t_scaled < ray.t_far * sum_e))
            || (sum_e > 0. && (t_scaled <= 0. || t_scaled > ray.t_far * sum_e))
        {
            return None;
        };

        let inv_sum_e = 1. / sum_e;
        // The time of the intersection:
        let t = t_scaled * inv_sum_e;

        if t <= 0. {
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
            self.uvs(mesh)
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
            pmath::coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
        } else {
            // Solve the system:
            let dpdu = (dp02.scale(duv12[1]) - dp12.scale(duv02[1])).scale(inv_det);
            let dpdv = (dp02.scale(-duv12[0]) + dp12.scale(duv02[0])).scale(inv_det);
            if dpdu.cross(dpdv).length2() == 0. {
                pmath::coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
            } else {
                (dpdu, dpdv)
            }
        };

        // TODO: texture stuff goes here

        // Calculate the shading normals now:
        let sn = if mesh.has_nrm() {
            n // No normal information was provided, so we use the calculated normal.
        } else {
            let norms = self.nrm(mesh);
            let sn = norms[0].scale(b[0]) + norms[1].scale(b[1]) + norms[2].scale(b[2]);
            if sn.length2() == 0. {
                n
            } else {
                sn.normalize()
            }
        };
        // Update n with the new shading normal from the provided normal:
        let n = pmath::align(sn, n);

        // Calculate the shading dndu and dndv values:
        let (sdndu, sdndv) = if mesh.has_nrm() {
            (Vec3::zero(), Vec3::zero())
        } else {
            let norms = self.nrm(mesh);
            let dn02 = norms[0] - norms[2];
            let dn12 = norms[1] - norms[2];

            if is_degen_uv {
                let dn = (norms[2] - norms[0]).cross(norms[1] - norms[0]);
                if dn.length2() == 0. {
                    (Vec3::zero(), Vec3::zero())
                } else {
                    pmath::coord_system(dn)
                }
            } else {
                let dndu = (dn02.scale(duv12[1]) - dn12.scale(duv02[1])).scale(inv_det);
                let dndv = (dn02.scale(-duv12[0]) + dn12.scale(duv02[0])).scale(inv_det);
                (dndu, dndv)
            }
        };

        // Calculate the shading tangents:
        let sdpdu = if mesh.has_tan() {
            let tans = self.tan(mesh);
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
        let (sdpdu, sdpdv) = {
            let sbt = sn.cross(sdpdu);
            if sbt.length2() > 0. {
                (sbt.cross(sdpdu), sbt.normalize())
            } else {
                pmath::coord_system(sn)
            }
        };

        let wo = -ray.dir;

        let geom_surf = GeomSurf {
            uv,
            dpdu,
            dpdv,
            sn,
            sdpdu,
            sdpdv,
            sdndu,
            sdndv,
        };

        Some(Surface {
            p,
            n,
            wo,
            t,
            time: ray.time,
            surf_type: SurfType::Geom(geom_surf),
        })
    }

    fn get_bbox(&self, mesh: &MeshData) -> BBox3<f64> {
        let poss = self.pos(mesh);
        BBox3::from_pnts(poss[0], poss[1]).combine_pnt(poss[2])
    }
}

// This represents the raw data that belongs to a mesh and gets passed to the triangle to
struct MeshData {
    pub triangles: Vec<Triangle>,
    pub pos: Vec<Vec3<f32>>,
    pub nrm: Vec<Vec3<f32>>,
    pub tan: Vec<Vec3<f32>>,
    pub uvs: Vec<Vec2<f32>>,
}

impl MeshData {
    fn has_nrm(&self) -> bool {
        !self.nrm.is_empty()
    }

    fn has_tan(&self) -> bool {
        !self.tan.is_empty()
    }

    fn has_uvs(&self) -> bool {
        !self.uvs.is_empty()
    }
}

pub struct Mesh {
    // The mesh data of the mesh.
    mesh_data: MeshData,
    // The bvh of the mesh.
    bvh: BVH<Triangle>,
    // The surface area of the mesh.
    surface_area: f64,
}

impl Mesh {
    /// Constructs a new mesh given all of the necessary data.
    pub fn new(
        triangles: Vec<Triangle>,
        pos: Vec<Vec3<f32>>,
        nrm: Vec<Vec3<f32>>,
        tan: Vec<Vec3<f32>>,
        uvs: Vec<Vec2<f32>>,
        max_triangles_per_leaf: usize,
    ) -> Self {
        let mesh_data = MeshData {
            triangles,
            pos,
            nrm,
            tan,
            uvs,
        };
        let bvh = BVH::new(&triangles, max_triangles_per_leaf, &mesh_data);

        Mesh {
            mesh_data,
            bvh,
            surface_area: -1.0,
        }
    }
}

impl Geometry for Mesh {
    fn intersect(&self, ray: Ray<f64>) -> Option<Surface> {
        // Calculate the ray information:
        self.bvh.intersect(ray, &self.mesh_data)
    }

    fn intersect_test(&self, ray: Ray<f64>) -> bool {
        self.bvh.intersect_test(ray, &self.mesh_data)
    }

    fn get_surface_area(&self) -> f64 {
        self.surface_area
    }

    /// Calculates the surface area of the specific mesh.
    fn calc_surface_area(&mut self) -> f64 {
        if self.surface_area >= 0.0 {
            return self.surface_area;
        }

        //let mesh_ref = self.get_ref();
        self.surface_area = self
            .mesh_data
            .triangles
            .iter()
            .fold(0.0, |sa, triangle| sa + triangle.area(&self.mesh_data));
        self.surface_area
    }

    fn get_bbox(&self) -> BBox3<f64> {
        self.bvh.get_bbox()
    }
}
