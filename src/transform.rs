use crate::math::bbox::BBox3;
use crate::math::matrix::{Mat3, Mat4};
use crate::math::quaternion::Quat;
use crate::math::ray::{Ray, RayDiff};
use crate::math::util::gamma_f64;
use crate::math::vector::{Vec3, Vec4};
use crate::geometry::Interaction;

use std::f64;

// A transform is a simple trait that has an interpolate function so that
// we can get a static transform as a result to perform operations on:
pub trait Transform {
    // We need to be able to interpolate it:
    fn interpolate(&self, t: f64) -> StaticTransform;
    // And, we need for it to be able to transform in this manner.
    // The reason is that we are bounding the motion of the box.
    fn bbox(&self, b: BBox3<f64>, t: f64) -> BBox3<f64>;
}

#[derive(Clone, Copy)]
pub struct StaticTransform {
    mat: Mat4<f64>,
    inv: Mat4<f64>,
}

impl StaticTransform {
    // Because a matrix could potentially not be invertible,
    // there is no gaurantee this will work:
    pub fn new(mat: Mat4<f64>) -> Option<Self> {
        let inv = match mat.inverse() {
            Some(i) => i,
            _ => return None,
        };
        Some(StaticTransform { mat, inv })
    }

    // Create a StaticTransform from a bunch of common ones:

    pub fn new_identity() -> Self {
        StaticTransform {
            mat: Mat4::new_identity(),
            inv: Mat4::new_identity(),
        }
    }

    pub fn new_translate(trans: Vec3<f64>) -> Self {
        StaticTransform {
            mat: Mat4::new_translate(trans),
            inv: Mat4::new_translate(-trans),
        }
    }

    pub fn new_scale(scale: Vec3<f64>) -> Self {
        StaticTransform {
            mat: Mat4::new_scale(scale),
            inv: Mat4::new_scale(scale.inv_scale(1.)),
        }
    }

    pub fn new_rotate(deg: f64, axis: Vec3<f64>) -> Self {
        let mat = Mat4::new_rotate(deg, axis);
        // inverse of rotation matrix is transpose
        let inv = mat.transpose();
        StaticTransform { mat, inv }
    }

    // Returns the normal matrix:
    pub fn get_mat(self) -> Mat4<f64> {
        self.mat
    }

    pub fn inverse(&self) -> Self {
        StaticTransform {
            mat: self.inv,
            inv: self.mat,
        }
    }

    pub fn point(self, p: Vec3<f64>) -> Vec3<f64> {
        let homog_p = Vec4::from_vec3(p, 1.);
        let homog_r = self.mat.mul_vec(homog_p);
        Vec3::from_vec4(homog_r)
    }

    pub fn normal(&self, n: Vec3<f64>) -> Vec3<f64> {
        let homog_n = Vec4::from_vec3(n, 0.);
        let homog_r = self.inv.transpose().mul_vec(homog_n);
        Vec3::from_vec4(homog_r)
    }

    pub fn vector(self, v: Vec3<f64>) -> Vec3<f64> {
        let homog_v = Vec4::from_vec3(v, 0.);
        let homog_r = self.mat.mul_vec(homog_v);
        Vec3::from_vec4(homog_r)
    }

    pub fn ray(self, r: Ray<f64>) -> Ray<f64> {
        Ray { 
            org: self.point(r.dir),
            dir: self.vector(r.dir),
        }
    }

    pub fn ray_diff(self, r: RayDiff<f64>) -> RayDiff<f64> {
        RayDiff {
            rx: self.ray(r.rx),
            ry: self.ray(r.ry),
        }
    }

    pub fn interaction(self, i: Interaction) -> Interaction {
        Interaction {
            p: self.point(i.p).normalize(),
            n: self.normal(i.n),
            wo: self.vector(i.wo).normalize(),
            time: i.time,
            uv: i.uv,
            dpdu: self.vector(i.dpdu),
            dpdv: self.vector(i.dpdv),
            shading_n: self.normal(i.shading_n).normalize(),
            shading_dpdu: self.vector(i.shading_dpdu),
            shading_dpdv: self.vector(i.shading_dpdv),
            shading_dndu: self.normal(i.shading_dndu),
            shading_dndv: self.normal(i.shading_dndv),
        }
    }
}

impl Transform for StaticTransform {
    fn interpolate(&self, _: f64) -> StaticTransform {
        *self
    }

    fn bbox(&self, b: BBox3<f64>, _: f64) -> BBox3<f64> {
        // From Arvo 1990 Graphics Gems 1

        let pmin = Vec3::from_vec4(self.mat.get_column(3));
        let pmax = pmin;

        let rot = Mat3::from_mat4(self.mat);

        let a0 = rot.get_column(0) * b.pmin;
        let a1 = rot.get_column(0) * b.pmax;
        let pmin = pmin + a0.min(a1);
        let pmax = pmax + a0.max(a1);

        let a0 = rot.get_column(1) * b.pmin;
        let a1 = rot.get_column(1) * b.pmax;
        let pmin = pmin + a0.min(a1);
        let pmax = pmax + a0.max(a1);

        let a0 = rot.get_column(2) * b.pmin;
        let a1 = rot.get_column(2) * b.pmax;
        let pmin = pmin + a0.min(a1);
        let pmax = pmax + a0.max(a1);

        BBox3 { pmin, pmax }
    }
}

#[derive(Clone, Copy)]
pub struct AnimatedTransform {
    start_transf: StaticTransform,
    end_transf: StaticTransform,

    start_time: f64,
    end_time: f64,

    // Decomposed information:
    start_trans: Vec3<f64>,
    end_trans: Vec3<f64>,

    start_rot: Quat<f64>,
    end_rot: Quat<f64>,

    start_scale: Mat4<f64>,
    end_scale: Mat4<f64>,

    // Knowing this can help with performance problems we may get:
    has_rot: bool,
}

impl AnimatedTransform {
    // This number is used when computing the bounding box transformation:
    const NUM_BOUND_SAMPLES: usize = 32;

    pub fn new(
        start_transf: StaticTransform,
        end_transf: StaticTransform,
        start_time: f64,
        end_time: f64,
    ) -> Self {
        // Because both start_transf and end_transf are invertible, their decomposition should also
        // be invertible.
        let (start_trans, start_rot, start_scale) =
            Self::decompose(start_transf.get_mat()).unwrap();
        let (end_trans, end_rot, end_scale) = Self::decompose(end_transf.get_mat()).unwrap();

        let end_rot = if start_rot.dot(end_rot).is_negative() {
            -end_rot
        } else {
            end_rot
        };
        // Check if the quaternions rotate enough for us to care. This is the constant
        // used in pbrt, so it should be fine here:
        let has_rot = start_rot.dot(end_rot) < 0.9995;

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
            has_rot,
        }
    }

    // Given a matrix, this will decompose it into a translation, rotation, and scale component.
    // Because some matrices are not invertible, it returns an option:
    fn decompose(mat: Mat4<f64>) -> Option<(Vec3<f64>, Quat<f64>, Mat4<f64>)> {
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
            let r3 = Vec4 {
                x: mat[3][0],
                y: mat[3][1],
                z: mat[3][2],
                w: 0.,
            };

            Mat4::new([r0, r1, r2, r3])
        };

        // Polar decomposition:
        let mut count = 0u32; // we want to limit the number of times we perform this operation
        let mut norm = f64::INFINITY; // so that we get at least one iteration
        let mut r_mat = upper_mat; // represents rotation and scale

        while count < 100 && norm > 0.0001 {
            let r_next = match r_mat.transpose().inverse() {
                Some(mat) => mat.scale(0.5),
                _ => return None,
            };

            let n0 = (r_mat[0] - r_next[0]).abs().horizontal_add();
            let n1 = (r_mat[1] - r_next[1]).abs().horizontal_add();
            let n2 = (r_mat[2] - r_next[2]).abs().horizontal_add();

            norm = n0.max(n1.max(n2));
            r_mat = r_next;
            count += 1;
        }

        let rot = Quat::from_mat4(r_mat);
        let scale = match r_mat.inverse() {
            Some(mat) => mat * upper_mat,
            _ => return None,
        };

        Some((trans, rot, scale))
    }
}

impl Transform for AnimatedTransform {
    fn interpolate(&self, t: f64) -> StaticTransform {
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
            let transf_mat = Mat4::new_translate(trans) * rot.to_mat4() * scale;
            StaticTransform::new(transf_mat).unwrap()
        }
    }

    fn bbox(&self, b: BBox3<f64>, _: f64) -> BBox3<f64> {
        // If there is no rotation, we can just do this:
        if !self.has_rot {
            self.start_transf
                .bbox(b, 0.)
                .combine_bnd(self.end_transf.bbox(b, 0.))
        } else {
            // I could do what pbrt does, but I'm too lazy. This bound transform should
            // only get called once in the preprocess step anyways, so it would only get called once.
            // This should be robust enough to handle most everything:
            let mut final_bbox = b;
            for i in 1..=Self::NUM_BOUND_SAMPLES {
                let t = (i as f64) / (Self::NUM_BOUND_SAMPLES as f64);
                let dt = (1. - t) * self.start_time + t * self.end_time;
                final_bbox = self.interpolate(dt).bbox(b, 0.).combine_bnd(final_bbox);
            }
            final_bbox
        }
    }
}
