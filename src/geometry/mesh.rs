use crate::geometry::{Interaction, RayIntInfo};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::util;
use crate::math::vector::{Vec2, Vec3};
use crate::transform::AnimatedTransform;

use std::cell::Cell;

// Triangle specifically for loading information and whatnot.
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub indices: [u32; 3], // Indices into a triangle
    pub attribute_id: u32, // The upper bits are a material id, the lower bits are a attribute pointer
}

impl Triangle {
    pub fn surface_area(self, mesh: &Mesh) -> f64 {
        // Get the attribute associated with the triangle:
        let attrib = {
            let attribute_index = self.attribute_id as usize;
            unsafe { mesh.attributes.get_unchecked(attribute_index) }
        };

        // If we already cached it, then just return that:
        if let Some(sa) = attrib.surface_area.get() {
            return sa;
        }

        let end_pos = attrib.triangle_start + attrib.num_triangles;
        let attrib_triangles =
            unsafe { mesh.triangles.get_unchecked(attrib.triangle_start..end_pos) };
        let sa = attrib_triangles
            .iter()
            .fold(0.0, |sa, triangle| sa + triangle.calc_area(mesh));
        attrib.surface_area.set(Some(sa));
        sa
    }

    // Calculates the surface area of a specific triangle:
    pub fn calc_area(self, mesh: &Mesh) -> f64 {
        let pos = self.get_pos(mesh);
        let a = pos[1] - pos[0];
        let b = pos[2] - pos[0];
        a.cross(b).length() * 0.5
    }

    // Calculates the bounding box of a specific triangle:
    pub fn calc_bound(self, mesh: &Mesh) -> BBox3<f64> {
        let poss = self.get_pos(mesh);
        BBox3::from_pnts(poss[0], poss[1]).combine_pnt(poss[2])
    }

    pub fn calc_centroid(self, mesh: &Mesh) -> Vec3<f64> {
        let pos = self.get_pos(mesh);
        (pos[0] + pos[1] + pos[2]).scale(1. / 3.)
    }

    // The ray should be in the same space as the triangle:
    pub fn intersect_test(&self, ray: Ray<f64>, int_info: RayIntInfo, mesh: &Mesh) -> bool {
        let poss = self.get_pos(mesh);

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
        if (sum_e < 0. && (t_scaled >= 0. || t_scaled < ray.max_t * sum_e))
            || (sum_e > 0. && (t_scaled <= 0. || t_scaled > ray.max_t * sum_e))
        {
            return false;
        };

        let inv_sum_e = 1. / sum_e;
        // The t of the intersection (make sure it's positive):
        t_scaled * inv_sum_e > 0.
    }

    // The ray should be in the same space as the triangle:
    pub fn intersect(
        &self,
        ray: Ray<f64>,
        int_info: RayIntInfo,
        mesh: &Mesh,
    ) -> Option<Interaction> {
        let poss = self.get_pos(mesh);

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
        if (sum_e < 0. && (t_scaled >= 0. || t_scaled < ray.max_t * sum_e))
            || (sum_e > 0. && (t_scaled <= 0. || t_scaled > ray.max_t * sum_e))
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

        // Get the attribute associated with the triangle:
        let attrib = {
            let attribute_index = self.attribute_id as usize;
            unsafe { mesh.attributes.get_unchecked(attribute_index) }
        };

        // Get the UV coordinates:
        let uvs = if attrib.has_uvs() {
            unsafe { self.get_uvs(attrib) }
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
            util::coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
        } else {
            // Solve the system:
            let dpdu = (dp02.scale(duv12[1]) - dp12.scale(duv02[1])).scale(inv_det);
            let dpdv = (dp02.scale(-duv12[0]) + dp12.scale(duv02[0])).scale(inv_det);
            if dpdu.cross(dpdv).length2() == 0. {
                util::coord_system((poss[2] - poss[0]).cross(poss[1] - poss[0]))
            } else {
                (dpdu, dpdv)
            }
        };

        // TODO: texture stuff goes here

        // Calculate the shading normals now:
        let (shading_n, shading_dndu, shading_dndv) = if attrib.has_nrm() {
            // Calculate the shading normal:
            let norms = unsafe { self.get_nrm(attrib) };
            let sn = norms[0].scale(b[0]) + norms[1].scale(b[1]) + norms[2].scale(b[2]);
            let shading_n = if sn.length2() == 0. {
                n
            } else {
                sn.normalize()
            };

            let dn02 = norms[0] - norms[2];
            let dn12 = norms[1] - norms[2];

            let (shading_dndu, shading_dndv) = if is_degen_uv {
                let dn = (norms[2] - norms[0]).cross(norms[1] - norms[0]);
                if dn.length2() == 0. {
                    (Vec3::zero(), Vec3::zero())
                } else {
                    util::coord_system(dn)
                }
            } else {
                let dndu = (dn02.scale(duv12[1]) - dn12.scale(duv02[1])).scale(inv_det);
                let dndv = (dn02.scale(-duv12[0]) + dn12.scale(duv02[0])).scale(inv_det);
                (dndu, dndv)
            };

            (shading_n, shading_dndu, shading_dndv)
        } else {
            (n, Vec3::zero(), Vec3::zero())
        };

        // Update n with the new shading normal from the provided normal:
        let n = util::align(shading_n, n);

        // Calculate the shading tangents:
        let shading_dpdu = if attrib.has_tan() {
            let tans = unsafe { self.get_tan(attrib) };
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
                util::coord_system(shading_n)
            }
        };

        let wo = -ray.dir;

        Some(Interaction {
            p,
            n,
            wo,
            t,
            uv,
            dpdu,
            dpdv,
            shading_n,
            shading_dpdu,
            shading_dpdv,
            shading_dndu,
            shading_dndv,
            attribute_id: self.attribute_id,
            mesh_id: mesh.id,
        })
    }

    fn get_pos(self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        unsafe {
            [
                mesh.pos.get_unchecked(self.indices[0] as usize).to_f64(),
                mesh.pos.get_unchecked(self.indices[1] as usize).to_f64(),
                mesh.pos.get_unchecked(self.indices[2] as usize).to_f64(),
            ]
        }
    }

    // Unsafe because it's not gauranteed that tangents are present.
    unsafe fn get_tan(self, attrib: &Attribute) -> [Vec3<f64>; 3] {
        [
            attrib.tan.get_unchecked(self.indices[0] as usize).to_f64(),
            attrib.tan.get_unchecked(self.indices[1] as usize).to_f64(),
            attrib.tan.get_unchecked(self.indices[2] as usize).to_f64(),
        ]
    }

    // Unsafe because it's not gauranteed that normals are present.
    unsafe fn get_nrm(self, attrib: &Attribute) -> [Vec3<f64>; 3] {
        // For performance reasons no check is applied here.
        [
            attrib.nrm.get_unchecked(self.indices[0] as usize).to_f64(),
            attrib.nrm.get_unchecked(self.indices[1] as usize).to_f64(),
            attrib.nrm.get_unchecked(self.indices[2] as usize).to_f64(),
        ]
    }

    // Unsafe because it's not gauranteed that UVs are present.
    unsafe fn get_uvs(self, attrib: &Attribute) -> [Vec2<f64>; 3] {
        // For performance reasons no check is applied here.
        [
            attrib.uvs.get_unchecked(self.indices[0] as usize).to_f64(),
            attrib.uvs.get_unchecked(self.indices[1] as usize).to_f64(),
            attrib.uvs.get_unchecked(self.indices[2] as usize).to_f64(),
        ]
    }
}

// Attributes that belong to a collection (such as surface area and whatnot):
#[derive(Clone, Debug)]
pub struct Attribute {
    // The triangles that this attribute is for:
    triangle_start: usize,
    num_triangles: usize,

    // The attributes themselves:
    nrm: Vec<Vec3<f32>>,
    tan: Vec<Vec3<f32>>,
    uvs: Vec<Vec2<f32>>,

    id: u32,

    // The surface area of the attribute:
    surface_area: Cell<Option<f64>>,
}

impl Attribute {
    pub fn new(
        triangle_start: usize,
        num_triangles: usize,
        nrm: Vec<Vec3<f32>>,
        tan: Vec<Vec3<f32>>,
        uvs: Vec<Vec2<f32>>,
        id: u32,
    ) -> Self {
        Attribute {
            triangle_start,
            num_triangles,
            nrm,
            tan,
            uvs,
            id,
            surface_area: Cell::new(None),
        }
    }
}

impl Attribute {
    pub fn has_nrm(&self) -> bool {
        !self.nrm.is_empty()
    }

    pub fn has_tan(&self) -> bool {
        !self.tan.is_empty()
    }

    pub fn has_uvs(&self) -> bool {
        !self.uvs.is_empty()
    }
}

// A single mesh
#[derive(Clone)]
pub struct Mesh {
    pub pos: Vec<Vec3<f32>>,
    pub triangles: Vec<Triangle>,
    attributes: Vec<Attribute>,

    // The transformation associated with mesh:
    transform: AnimatedTransform,

    id: u32,
}

// Mesh access is done through u32 values to save on storage:
impl Mesh {
    pub fn new(
        pos: Vec<Vec3<f32>>,
        triangles: Vec<Triangle>,
        attributes: Vec<Attribute>,
        transform: AnimatedTransform,
        id: u32,
    ) -> Self {
        let mut result = Mesh {
            pos,
            triangles,
            attributes,
            transform,
            id,
        };

        // Check if it's animated. If it isn't, we can easily "cache" the transformation
        // by applying them now:
        if !transform.is_animated() {
            let transf = transform.interpolate(0.0);
            transf.points_f32(&mut result.pos);
            for attribute in result.attributes.iter_mut() {
                transf.vectors_f32(&mut attribute.tan);
                transf.normals_f32(&mut attribute.nrm);
            }
        }

        result
    }

    pub fn get_transform(&self) -> AnimatedTransform {
        self.transform
    }

    pub fn get_attribute(&self, attribute_id: u32) -> &Attribute {
        &self.attributes[attribute_id as usize]
    }

    pub fn get_attribute_mut(&mut self, attribute_id: u32) -> &mut Attribute {
        &mut self.attributes[attribute_id as usize]
    }
}
