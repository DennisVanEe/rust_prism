use crate::geometry::GeomInteraction;
use crate::math::bbox::BBox3;
use crate::math::matrix::{Mat3, Mat4};
use crate::math::quaternion::Quat;
use crate::math::ray::{Ray, RayDiff};
use crate::math::vector::{Vec3, Vec4};

use std::f32;
use std::ops::Mul;

#[derive(Clone, Copy)]
pub struct Transform {
    mat: Mat4<f32>,
    inv: Mat4<f32>,
}

impl Transform {
    // Because a matrix could potentially not be invertible,
    // there is no gaurantee this will work:
    pub fn from_matrix(mat: Mat4<f32>) -> Option<Self> {
        let inv = match mat.inverse() {
            Some(i) => i,
            _ => return None,
        };
        Some(Transform { mat, inv })
    }

    // Create a StaticTransform from a bunch of common ones:

    pub fn identity() -> Self {
        Transform {
            mat: Mat4::new_identity(),
            inv: Mat4::new_identity(),
        }
    }

    pub fn translate(trans: Vec3<f32>) -> Self {
        Transform {
            mat: Mat4::new_translate(trans),
            inv: Mat4::new_translate(-trans),
        }
    }

    pub fn scale(scale: Vec3<f32>) -> Self {
        Transform {
            mat: Mat4::new_scale(scale),
            inv: Mat4::new_scale(scale.inv_scale(1.)),
        }
    }

    pub fn rotate(deg: f32, axis: Vec3<f32>) -> Self {
        let mat = Mat4::new_rotate(deg, axis);
        // inverse of rotation matrix is transpose
        let inv = mat.transpose();
        Transform { mat, inv }
    }

    // Note that fov is in degrees
    pub fn perspective(fov: f32, near: f32, far: f32) -> Self {
        let perspective = Mat4::new([
            Vec4 {
                x: 1.,
                y: 0.,
                z: 0.,
                w: 0.,
            },
            Vec4 {
                x: 0.,
                y: 1.,
                z: 0.,
                w: 0.,
            },
            Vec4 {
                x: 0.,
                y: 0.,
                z: far / (far - near),
                w: -far * near / (far - near),
            },
            Vec4 {
                x: 0.,
                y: 0.,
                z: 1.,
                w: 0.,
            },
        ]);
        // Calculate the FOV information:
        let inv_tan_angle = 1. / (fov.to_radians() / 2.).tan();
        // Calculate the scale used:
        Self::scale(Vec3 {
            x: inv_tan_angle,
            y: inv_tan_angle,
            z: 1.,
        }) * Self::from_matrix(perspective).unwrap()
    }

    /// Inverses the transformation
    pub fn inverse(&self) -> Self {
        Transform {
            mat: self.inv,
            inv: self.mat,
        }
    }

    // Returns the normal matrix:
    pub fn get_mat(self) -> Mat4<f32> {
        self.mat
    }

    pub fn point(self, p: Vec3<f32>) -> Vec3<f32> {
        self.mat.mul_vec_one(p)
    }

    pub fn points(self, ps: &mut [Vec3<f32>]) {
        for p in ps.iter_mut() {
            *p = self.point(*p);
        }
    }

    pub fn proj_point(self, p: Vec3<f32>) -> Vec3<f32> {
        let homog_p = Vec4::from_vec3(p, 1.);
        let homog_r = self.mat.mul_vec(homog_p);
        Vec3::from_vec4(homog_r).scale(1. / homog_r.w)
    }

    pub fn proj_points(self, ps: &mut [Vec3<f32>]) {
        for p in ps.iter_mut() {
            *p = self.proj_point(*p);
        }
    }

    pub fn normal(self, n: Vec3<f32>) -> Vec3<f32> {
        self.inv.transpose().mul_vec_zero(n)
    }

    pub fn normals(self, ns: &mut [Vec3<f32>]) {
        let mat = self.inv.transpose();
        for n in ns.iter_mut() {
            *n = mat.mul_vec_zero(*n);
        }
    }

    pub fn vector(self, v: Vec3<f32>) -> Vec3<f32> {
        self.mat.mul_vec_zero(v)
    }

    pub fn vectors(self, vs: &mut [Vec3<f32>]) {
        for v in vs.iter_mut() {
            *v = self.mat.mul_vec_zero(*v);
        }
    }
}

// Given a matrix, this will decompose it into a translation, rotation, and scale component.
// Because some matrices are not invertible, it returns an option.
// The decomposition is such that: p' = (TRS)p
pub fn decompose(mat: Mat4<f32>) -> Option<(Vec3<f32>, Quat<f32>, Mat4<f32>)> {
    let trans = Vec3::from_vec4(mat.get_column(3));

    // keep the rotational information that we are interested
    // in this case (the RS component):
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
    let mut norm = f32::INFINITY; // so that we get at least one iteration
    let mut r_mat = upper_mat; // represents rotation and scale (RS)

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

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Transform {
            mat: self.mat * rhs.mat,
            inv: self.inv * rhs.inv,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AnimatedTransform {
    start_transf: Transform,
    end_transf: Transform,

    start_time: f32,
    end_time: f32,

    // Decomposed information:
    start_trans: Vec3<f32>,
    end_trans: Vec3<f32>,

    start_rot: Quat<f32>,
    end_rot: Quat<f32>,

    start_scale: Mat4<f32>,
    end_scale: Mat4<f32>,

    // Knowing this can help with performance problems we may get:
    has_rot: bool,
}

impl AnimatedTransform {
    pub fn new(
        start_transf: Transform,
        end_transf: Transform,
        start_time: f32,
        end_time: f32,
    ) -> Self {
        // Because both start_transf and end_transf are invertible, their decomposition should also
        // be invertible.
        let (start_trans, start_rot, start_scale) =
            Self::decompose(start_transf.get_mat()).unwrap();
        let (end_trans, end_rot, end_scale) = Self::decompose(end_transf.get_mat()).unwrap();

        let end_rot = if start_rot.dot(end_rot).is_sign_negative() {
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
    fn decompose(mat: Mat4<f32>) -> Option<(Vec3<f32>, Quat<f32>, Mat4<f32>)> {
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
        let mut norm = f32::INFINITY; // so that we get at least one iteration
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

    fn interpolate(&self, t: f32) -> Transform {
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
            Transform::from_matrix(transf_mat).unwrap()
        }
    }
}