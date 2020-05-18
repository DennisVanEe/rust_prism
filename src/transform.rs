use crate::math::matrix::Mat3x4;
use crate::math::quaternion::Quat;
use crate::math::ray::{Ray, RayDiff};
use crate::math::vector::{Vec3, Vec4};
use crate::mesh::Interaction;

use std::ops::Mul;

#[derive(Clone, Copy)]
pub struct AnimatedTransform {
    start_transf: Transform,
    end_transf: Transform,

    start_time: f64,
    end_time: f64,

    // Decomposed information:
    start_trans: Vec3<f64>,
    end_trans: Vec3<f64>,

    start_rot: Quat<f64>,
    end_rot: Quat<f64>,

    start_scale: Mat3x4<f64>,
    end_scale: Mat3x4<f64>,

    // Knowing this can help with performance problems we may get:
    has_rot: bool,
    animated: bool,
}

impl AnimatedTransform {
    pub fn new(
        start_transf: Transform,
        end_transf: Transform,
        start_time: f64,
        end_time: f64,
    ) -> Self {
        // Because both start_transf and end_transf are invertible, their decomposition should also
        // be invertible.
        let (start_trans, start_rot, start_scale) = Self::decompose(start_transf.get_mat());
        let (end_trans, end_rot, end_scale) = Self::decompose(end_transf.get_mat());

        let end_rot = if start_rot.dot(end_rot).is_sign_negative() {
            -end_rot
        } else {
            end_rot
        };

        AnimatedTransform {
            start_transf,
            end_transf,
            start_time,
            end_time,
            start_trans,
            end_trans,
            start_rot,
            end_rot,
            start_scale,
            end_scale,
            has_rot: start_rot.dot(end_rot) < 0.9995,
            animated: start_time == end_time,
        }
    }

    pub fn is_animated(self) -> bool {
        self.animated
    }

    // Given a matrix, this will decompose it into a translation, rotation, and scale component.
    // Because some matrices are not invertible, it returns an option:
    fn decompose(mat: Mat3x4<f64>) -> (Vec3<f64>, Quat<f64>, Mat3x4<f64>) {
        let trans = Vec3::from_vec4(mat.get_column(3));

        // keep the rotational information that we are interested
        // in this case:
        let upper_mat = {
            let r0 = Vec4 {
                x: mat[0][0],
                y: mat[0][1],
                z: mat[0][2],
                w: 0.,
            };
            let r1 = Vec4 {
                x: mat[1][0],
                y: mat[1][1],
                z: mat[1][2],
                w: 0.,
            };
            let r2 = Vec4 {
                x: mat[2][0],
                y: mat[2][1],
                z: mat[2][2],
                w: 0.,
            };

            Mat3x4::new([r0, r1, r2])
        };

        // Polar decomposition:
        let mut count = 0u32; // we want to limit the number of times we perform this operation
        let mut norm = f64::INFINITY; // so that we get at least one iteration
        let mut r_mat = upper_mat; // represents rotation and scale

        while count < 100 && norm > 0.0001 {
            let r_next = r_mat.transpose().inverse().scale(0.5);

            let n0 = (r_mat[0] - r_next[0]).abs().horizontal_add();
            let n1 = (r_mat[1] - r_next[1]).abs().horizontal_add();
            let n2 = (r_mat[2] - r_next[2]).abs().horizontal_add();

            norm = n0.max(n1.max(n2));
            r_mat = r_next;
            count += 1;
        }

        let rot = Quat::from_mat3x4(r_mat);
        let scale = r_mat.inverse() * upper_mat;

        (trans, rot, scale)
    }

    pub fn interpolate(self, t: f64) -> Transform {
        // Check if the time is out of bounds, in which case we just return
        // the transform we care about:
        if t <= self.start_time {
            self.start_transf
        } else if t >= self.end_time {
            self.end_transf
        } else {
            let dt = (t - self.start_time) / (self.end_time - self.start_time);
            let trans = self.start_trans.lerp(self.end_trans, dt);
            let rot = self.start_rot.slerp(self.end_rot, dt);
            let scale = self.start_scale.lerp(self.end_scale, dt);

            // We can use unwrapped because we know for a fact that the matrix
            // is invertible:
            let transf_mat = Mat3x4::new_translate(trans) * rot.to_mat3x4() * scale;
            Transform::from_matrix(transf_mat)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    mat: Mat3x4<f64>,
    inv: Mat3x4<f64>,
}

impl Transform {
    // Because a matrix could potentially not be invertible,
    // there is no gaurantee this will work:
    pub fn from_matrix(mat: Mat3x4<f64>) -> Self {
        let inv = mat.inverse();
        Transform { mat, inv }
    }

    // Create a StaticTransform from a bunch of common ones:

    pub fn new_identity() -> Self {
        Transform {
            mat: Mat3x4::new_identity(),
            inv: Mat3x4::new_identity(),
        }
    }

    pub fn new_translate(trans: Vec3<f64>) -> Self {
        Transform {
            mat: Mat3x4::new_translate(trans),
            inv: Mat3x4::new_translate(-trans),
        }
    }

    pub fn new_scale(scale: Vec3<f64>) -> Self {
        Transform {
            mat: Mat3x4::new_scale(scale),
            inv: Mat3x4::new_scale(scale.inv_scale(1.)),
        }
    }

    pub fn new_rotate(deg: f64, axis: Vec3<f64>) -> Self {
        let mat = Mat3x4::new_rotate(deg, axis);
        // inverse of rotation matrix is transpose
        let inv = mat.transpose();
        Transform { mat, inv }
    }

    /// Inverses the transformation
    pub fn inverse(&self) -> Self {
        Transform {
            mat: self.inv,
            inv: self.mat,
        }
    }

    // Returns the normal matrix:
    pub fn get_mat(self) -> Mat3x4<f64> {
        self.mat
    }

    pub fn get_inv(self) -> Mat3x4<f64> {
        self.inv
    }

    pub fn point(self, p: Vec3<f64>) -> Vec3<f64> {
        self.mat.mul_vec_one(p)
    }

    pub fn points(self, ps: &mut [Vec3<f64>]) {
        for p in ps.iter_mut() {
            *p = self.point(*p);
        }
    }

    pub fn points_f32(self, ps: &mut [Vec3<f32>]) {
        for p in ps.iter_mut() {
            let pf64 = Vec3 {
                x: p.x as f64,
                y: p.y as f64,
                z: p.z as f64,
            };
            let pf64 = self.point(pf64);
            *p = Vec3 {
                x: pf64.x as f32,
                y: pf64.y as f32,
                z: pf64.z as f32,
            }
        }
    }

    pub fn normal(self, n: Vec3<f64>) -> Vec3<f64> {
        self.inv.transpose().mul_vec_zero(n)
    }

    pub fn normals(self, ns: &mut [Vec3<f64>]) {
        let mat = self.inv.transpose();
        for n in ns.iter_mut() {
            *n = mat.mul_vec_zero(*n);
        }
    }

    pub fn normals_f32(self, ns: &mut [Vec3<f32>]) {
        let mat = self.inv.transpose();
        for n in ns.iter_mut() {
            let nf64 = Vec3 {
                x: n.x as f64,
                y: n.y as f64,
                z: n.z as f64,
            };
            let nf64 = mat.mul_vec_zero(nf64);
            *n = Vec3 {
                x: nf64.x as f32,
                y: nf64.y as f32,
                z: nf64.z as f32,
            }
        }
    }

    pub fn vector(self, v: Vec3<f64>) -> Vec3<f64> {
        self.mat.mul_vec_zero(v)
    }

    pub fn vectors(self, vs: &mut [Vec3<f64>]) {
        for v in vs.iter_mut() {
            *v = self.mat.mul_vec_zero(*v);
        }
    }

    pub fn vectors_f32(self, vs: &mut [Vec3<f32>]) {
        for v in vs.iter_mut() {
            let vf64 = Vec3 {
                x: v.x as f64,
                y: v.y as f64,
                z: v.z as f64,
            };
            let vf64 = self.point(vf64);
            *v = Vec3 {
                x: vf64.x as f32,
                y: vf64.y as f32,
                z: vf64.z as f32,
            }
        }
    }

    pub fn interaction(self, i: Interaction) -> Interaction {
        Interaction {
            p: self.point(i.p),
            n: self.normal(i.n).normalize(),
            wo: self.vector(i.wo).normalize(),

            t: i.t,

            uv: i.uv,
            dpdu: self.vector(i.dpdu),
            dpdv: self.vector(i.dpdv),

            shading_n: self.normal(i.shading_n).normalize(),
            shading_dpdu: self.vector(i.shading_dpdu),
            shading_dpdv: self.vector(i.shading_dpdv),
            shading_dndu: self.normal(i.shading_dndu),
            shading_dndv: self.normal(i.shading_dndv),

            material_id: i.material_id,
        }
    }

    pub fn ray(self, r: Ray<f64>) -> Ray<f64> {
        Ray {
            org: self.point(r.dir),
            dir: self.vector(r.dir),
            time: r.time,
            t_far: r.t_far,
            t_near: r.t_near,
            ray_diff: match r.ray_diff {
                Some(diff) => Some(self.ray_diff(diff)),
                _ => None,
            },
        }
    }

    pub fn ray_diff(self, r: RayDiff<f64>) -> RayDiff<f64> {
        RayDiff {
            rx_org: self.point(r.rx_org),
            ry_org: self.point(r.ry_org),

            rx_dir: self.vector(r.rx_dir),
            ry_dir: self.vector(r.ry_dir),
        }
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Transform {
            mat: self.mat * rhs.mat,
            inv: self.inv * rhs.inv,
        }
    }
}
