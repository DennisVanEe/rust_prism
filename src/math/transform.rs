use crate::math::matrix::{Mat3, Mat4};
use crate::math::vector::{Vec3, Vec4};
use crate::math::quaternion::Quat;
use crate::math::bbox::BBox3;

use num_traits::{Bounded, Float, Signed};

use std::ops::Mul;

/// Transforms, being a massive 32 floats, can't be copied
/// willy nilly. To do that you have to clone it.
/// Transforms are always gauranteed to be invertible.
#[derive(Clone, Copy, Debug)]
pub struct Transform<T: Bounded + Float> {
    nrm: Mat4<T>,
    inv: Mat4<T>,
}

impl<T: Bounded + Float> Transform<T> {
    /// The function will perform the inversion itself.
    /// Note that, because the inverse can be undefined,
    /// it returns an optional.
    pub fn new(nrm: Mat4<T>) -> Option<Self> {
        let inv_opt = nrm.inverse();
        match inv_opt {
            Some(inv) => Some(Transform { nrm, inv }),
            _ => None,
        }
    }
    
    // generates translation for transform:
    pub fn translation(trans: Vec3<T>) -> Self {
        let nrm = Mat4::translation(trans);
        let inv = Mat4::translation(-trans);
        Transform { nrm, inv }
    }

    // generates the inverse of the transformation (just the old swap):
    pub fn inverse(&self) -> Self {
        let mat_copy = self.clone();
        Transform {
            nrm: mat_copy.inv,
            inv: mat_copy.nrm,
        }
    }

    pub fn transf_point(self, p: Vec3<T>) -> Vec3<T> {
        let pw = Vec4::from_vec3(p, T::one());
        let res = self.nrm.vec_mul(pw);
        Vec3::from_vec4(res)
    }

    // Transforms a bounding box (Arvo 1990 Graphics Gems 1):
    pub fn transf_bbox3(self, bbox: BBox3<T>) -> BBox3<T> {
        // Get the translation portion first:
        let pmin = Vec3::from_vec4(self.nrm.get_column(3));
        let pmax = pmin;

        let rot = Mat3::from_mat4(self.nrm);

        let a = rot.get_column(0) * bbox.pmin;
        let b = rot.get_column(0) * bbox.pmax;
        let pmin = pmin + a.min(b);
        let pmax = pmax + a.max(b);

        let a = rot.get_column(1) * bbox.pmin;
        let b = rot.get_column(1) * bbox.pmax;
        let pmin = pmin + a.min(b);
        let pmax = pmax + a.max(b);

        let a = rot.get_column(2) * bbox.pmin;
        let b = rot.get_column(2) * bbox.pmax;
        let pmin = pmin + a.min(b);
        let pmax = pmax + a.max(b);

        BBox3 { pmin, pmax }
    }
}

impl<T: Float> Mul for Transform<T> {
    type Output = Self;

    fn mul(self, o: Self) -> Self {
        Transform {
            nrm: self.nrm * o.nrm,
            inv: self.inv * o.inv,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnimatedTransform<T: Signed + Float> {
    start_transform: Transform<T>,
    end_transform: Transform<T>,
    
    start_time: T,
    end_time: T,

    // Decomposed information:
    start_trans: Vec3<T>,
    end_trans: Vec3<T>,

    start_rot: Quat<T>,
    end_rot: Quat<T>, // might not have rotation

    start_scale: Mat4<T>,
    end_scale: Mat4<T>,

    has_rot: bool,
    // A lot of the time, these objects won't be animated:
    is_animated: bool,
}

impl<T: Signed + Float> AnimatedTransform<T> {
    // mat must be an affine transformation
    pub fn new(start_mat: Mat4<T>, end_mat: Mat4<T>, start_time: T, end_time: T) -> Option<Self> {
        let (start_trans, start_rot, start_scale) = match Self::decompose(start_mat) {
            Some(result) => result,
            _ => return None,
        };

        let (end_trans, end_rot, end_scale) = match Self::decompose(end_mat) {
            Some(result) => result,
            _ => return None,
        };

        let end_rot = if start_rot.dot(end_rot).is_negative() { -end_rot } else { end_rot };
        let has_rot = start_rot.dot(end_rot) < T::from::<f32>(0.9995f32).unwrap();

        // This is code I just copied from pbrt...
        if has_rot {

        }
    }

    // If given a bounding box, will bound the entire motion of this bounding box
    pub fn motion_bound(self, bound: BBox3<T>) -> BBox3<T> {
        // If it isn't animated, then just transform the bound itself:
        if !self.is_animated {

        }
    }

    pub fn interpolate(self, time: T) -> Transform<T> {
        // Check if we even have an end transform:
        if !self.is_animated || time <= self.start_time {
            self.start_transform
        } else if time >= self.end_time {
            self.end_transform
        } else {
            let dt = (time - self.start_time) / (self.end_time - self.start_time);
            let trans = self.start_trans.lerp(self.end_trans, dt);
            let rot = self.start_rot.slerp(self.end_rot, dt);
            let scale = self.start_scale.lerp(self.end_scale, dt);

            let int_mat = Mat4::translation(trans) * rot.to_mat4() * scale;
            // This is part of the core code, I'm assuming we can always invert it,
            // so this should work:
            Transform::new(int_mat).unwrap()
        }
    }

    // Optional, as the matrix may not have an inverse, which leads to problems:
    fn decompose(mat: Mat4<T>) -> Option<(Vec3<T>, Quat<T>, Mat4<T>)> {
        let trans = Vec3::from_vec4(mat.get_column(3));

        // keep the rotational information that we are interested
        // in this case:
        let upper_mat = {
            let r0 = Vec4 {
                x: mat[0][0],
                y: mat[0][1],
                z: mat[0][2],
                w: T::zero(),
            };
            let r1 = Vec4 {
                x: mat[1][0],
                y: mat[1][1],
                z: mat[1][2],
                w: T::zero(),
            };
            let r2 = Vec4 {
                x: mat[2][0],
                y: mat[2][1],
                z: mat[2][2],
                w: T::zero(),
            };
            let r3 = Vec4 {
                x: mat[3][0],
                y: mat[3][1],
                z: mat[3][2],
                w: T::zero(),
            };

            Mat4::new([r0, r1, r2, r3])
        };

        // Polar decomposition:
        let mut count = 0u32; // we want to limit the number of times we perform this operation
        let mut norm = T::infinity(); // so that we get at least one iteration
        let mut r_mat = upper_mat; // represents rotation and scale

        while count < 100 && norm > T::from::<f64>(0.0001).unwrap() {
            // Update r_mat:
            let r_next = match r_mat.transpose().inverse() {
                Some(mat) => mat.scale(T::from::<f64>(0.5).unwrap()),
                _ => return None,
            };

            let n0 = (r_mat[0] - r_next[0]).abs().horizontal_add();
            let n1 = (r_mat[1] - r_next[1]).abs().horizontal_add();
            let n2 = (r_mat[2] - r_next[2]).abs().horizontal_add();
            
            norm = n0.max(n1.max(n2));
            r_mat = r_next;
        }

        let rot = Quat::from_mat4(r_mat);
        let scale = match r_mat.inverse() {
            Some(mat) => mat * upper_mat,
            _ => return None,
        };

        Some((trans, rot, scale))
    }
}
