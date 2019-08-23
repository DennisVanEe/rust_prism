// A n animated transform is one that animates between
// two static transformations.

use crate::transform::Transform;
use crate::transform::static_transform::StaticTransform;
use crate::math::matrix::{Mat3, Mat4};
use crate::math::vector::{Vec3, Vec4};
use crate::math::ray::Ray;
use crate::math::bbox::BBox3;

use std::f64;

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

    pub fn new(start_transf: StaticTransform, end_transf: StaticTransform, start_time: f64, end_time: f64) -> Self {
        let (start_trans, start_rot, start_scale) = match Self::decompose(start_transf.get_mat()) {
            Some(result) => result,
            _ => return None,
        };

        let (end_trans, end_rot, end_scale) = match Self::decompose(end_transf.get_mat()) {
            Some(result) => result,
            _ => return None,
        };
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
        let mut norm = f64::infinity(); // so that we get at least one iteration
        let mut r_mat = upper_mat; // represents rotation and scale

        while count < 100 && norm > T::from(0.0001).unwrap() {
            let r_next = match r_mat.transpose().inverse() {
                Some(mat) => mat.scale(T::from(0.5).unwrap()),
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