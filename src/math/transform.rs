use crate::math::matrix::{Mat3, Mat4};
use crate::math::vector::{Vec3, Vec4};
use crate::math::quaternion::Quat;
use crate::math::bbox::BBox3;

use num_traits::{Bounded, Float, Signed};

use std::ops::Mul;

// Transforms are a special class of functions that have to support the following operations:


// #[derive(Clone, Copy, Debug)]
// // A Transform needs to be bounded in order to handle transformations
// // on bboxes.
// pub struct Transform<T: Bounded + Float> {
//     nrm: Mat4<T>,
//     inv: Mat4<T>,
// }

// impl<T: Bounded + Float> Transform<T> {
//     /// The function will perform the inversion itself.
//     /// Note that, because the inverse can be undefined,
//     /// it returns an optional.
//     pub fn new(nrm: Mat4<T>) -> Option<Self> {
//         let inv_opt = nrm.inverse();
//         match inv_opt {
//             Some(inv) => Some(Transform { nrm, inv }),
//             _ => None,
//         }
//     }

//     pub fn new_identity() -> Self {
//         Transform {
//             nrm: Mat4::new_identity(),
//             inv: Mat4::new_identity(),
//         }
//     }
    
//     // generates translation for transform:
//     pub fn new_translate(trans: Vec3<T>) -> Self {
//         let nrm = Mat4::new_translate(trans);
//         let inv = Mat4::new_translate(-trans);
//         Transform { nrm, inv }
//     }

//     pub fn new_scale(scale: Vec3<T>) -> Self {
//         let nrm = Mat4::new_scale(scale);
//         let inv = Mat4::new_scale(scale.inv_scale(T::one()));
//         Transform { nrm, inv }
//     }

//     pub fn new_rotate(deg: T, axis: Vec3<T>) -> Self {
//         let nrm = Mat4::new_rotate(deg, axis);
//         let inv = nrm.transpose(); // inverse of rotation matrix is transpose
//         Transform { nrm, inv }
//     }

//     // generates the inverse of the transformation (just the old swap):
//     pub fn inverse(&self) -> Self {
//         let mat_copy = self.clone();
//         Transform {
//             nrm: mat_copy.inv,
//             inv: mat_copy.nrm,
//         }
//     }

//     pub fn transf_point(self, p: Vec3<T>) -> Vec3<T> {
//         let pw = Vec4::from_vec3(p, T::one());
//         let res = self.nrm.vec_mul(pw);
//         Vec3::from_vec4(res)
//     }

//     // Transforms a bounding box (Arvo 1990 Graphics Gems 1):
//     pub fn transf_bbox3(self, bbox: BBox3<T>) -> BBox3<T> {
//         // Get the translation portion first:
//         let pmin = Vec3::from_vec4(self.nrm.get_column(3));
//         let pmax = pmin;

//         let rot = Mat3::from_mat4(self.nrm);

//         let a = rot.get_column(0) * bbox.pmin;
//         let b = rot.get_column(0) * bbox.pmax;
//         let pmin = pmin + a.min(b);
//         let pmax = pmax + a.max(b);

//         let a = rot.get_column(1) * bbox.pmin;
//         let b = rot.get_column(1) * bbox.pmax;
//         let pmin = pmin + a.min(b);
//         let pmax = pmax + a.max(b);

//         let a = rot.get_column(2) * bbox.pmin;
//         let b = rot.get_column(2) * bbox.pmax;
//         let pmin = pmin + a.min(b);
//         let pmax = pmax + a.max(b);

//         BBox3 { pmin, pmax }
//     }

//     pub fn get_mat(self) -> Mat4<T> {
//         self.nrm
//     }

//     pub fn get_inv_mat(self) -> Mat4<T> {
//         self.inv
//     }
// }

// impl<T: Bounded + Float> Mul for Transform<T> {
//     type Output = Self;

//     fn mul(self, o: Self) -> Self {
//         Transform {
//             nrm: self.nrm * o.nrm,
//             inv: self.inv * o.inv,
//         }
//     }
// }

// #[derive(Clone, Copy, Debug)]
// pub struct AnimatedTransform<T: Bounded + Signed + Float> {
//     start_transf: Transform<T>,
//     end_transf: Transform<T>,
    
//     start_time: T,
//     end_time: T,

//     // Decomposed information:
//     start_trans: Vec3<T>,
//     end_trans: Vec3<T>,

//     start_rot: Quat<T>,
//     end_rot: Quat<T>, // might not have rotation

//     start_scale: Mat4<T>,
//     end_scale: Mat4<T>,

//     has_rot: bool,
//     // A lot of the time, these objects won't be animated:
//     is_animated: bool,
// }

// impl<T: Bounded + Signed + Float> AnimatedTransform<T> {
//     // Number of time samples to calculate the bounding box
//     // for the motion:
//     const NUM_BOUND_SAMPLES: usize = 32;

//     // mat must be an affine transformation.
//     // It returns an option because the matrix could potentially not be invertible
//     pub fn new(start_transf: Transform<T>, end_transf: Transform<T>, start_time: T, end_time: T) -> Option<Self> {
//         let (start_trans, start_rot, start_scale) = match Self::decompose(start_transf.get_mat()) {
//             Some(result) => result,
//             _ => return None,
//         };

//         let (end_trans, end_rot, end_scale) = match Self::decompose(end_transf.get_mat()) {
//             Some(result) => result,
//             _ => return None,
//         };

//         let end_rot = if start_rot.dot(end_rot).is_negative() { -end_rot } else { end_rot };
//         let has_rot = start_rot.dot(end_rot) < T::from(0.9995f32).unwrap();

//         Some(AnimatedTransform {
//             start_transf,
//             end_transf,
//             start_time,
//             end_time,
//             start_trans,
//             end_trans,
//             start_rot,
//             end_rot,
//             start_scale,
//             end_scale,
//             has_rot,
//             is_animated: true,
//         })
//     }

//     // If given a bounding box, will bound the entire motion of this bounding box.
//     // This could potentially be a very costly operation, so, try not to call this
//     // too often (really, one shouldn't be calling this very often).
//     pub fn motion_bound(self, bbox: BBox3<T>) -> BBox3<T> {
//         // These are the cases that are efficient to calculate:
//         if !self.is_animated {
//              self.start_transf.transf_bbox3(bbox)
//         } else if !self.has_rot {
//             self.start_transf.transf_bbox3(bbox).combine_bnd(self.end_transf.transf_bbox3(bbox))
//         } else {
//             // I could do what pbrt does, but I'm too lazy. This bound transform should
//             // only get called once in the preprocess step anyways, so it would only get called once.
//             // This should be robust enough to handle most everything:
//             let mut final_bbox = bbox;
//             for i in 1..=Self::NUM_BOUND_SAMPLES {
//                 let t = T::from(i).unwrap() / T::from(Self::NUM_BOUND_SAMPLES).unwrap();
//                 let dt = (T::one() - t) * self.start_time + t * self.end_time;
//                 final_bbox = self.interpolate(dt).transf_bbox3(bbox).combine_bnd(final_bbox);
//             }
//             final_bbox
//         }
//     }

//     pub fn interpolate(self, time: T) -> Transform<T> {
//         // Check if we even have an end transform:
//         if !self.is_animated || time <= self.start_time {
//             self.start_transf
//         } else if time >= self.end_time {
//             self.end_transf
//         } else {
//             let dt = (time - self.start_time) / (self.end_time - self.start_time);
//             let trans = self.start_trans.lerp(self.end_trans, dt);
//             let rot = self.start_rot.slerp(self.end_rot, dt);
//             let scale = self.start_scale.lerp(self.end_scale, dt);

//             let int_mat = Mat4::new_translate(trans) * rot.to_mat4() * scale;
//             // We already checked that it was invertible, so we don't have to check it
//             // it further down the line:
//             Transform::new(int_mat).unwrap()
//         }
//     }

//     // Optional, as the matrix may not have an inverse, which leads to problems:
//     fn decompose(mat: Mat4<T>) -> Option<(Vec3<T>, Quat<T>, Mat4<T>)> {
//         let trans = Vec3::from_vec4(mat.get_column(3));

//         // keep the rotational information that we are interested
//         // in this case:
//         let upper_mat = {
//             let r0 = Vec4 {
//                 x: mat[0][0],
//                 y: mat[0][1],
//                 z: mat[0][2],
//                 w: T::zero(),
//             };
//             let r1 = Vec4 {
//                 x: mat[1][0],
//                 y: mat[1][1],
//                 z: mat[1][2],
//                 w: T::zero(),
//             };
//             let r2 = Vec4 {
//                 x: mat[2][0],
//                 y: mat[2][1],
//                 z: mat[2][2],
//                 w: T::zero(),
//             };
//             let r3 = Vec4 {
//                 x: mat[3][0],
//                 y: mat[3][1],
//                 z: mat[3][2],
//                 w: T::zero(),
//             };

//             Mat4::new([r0, r1, r2, r3])
//         };

//         // Polar decomposition:
//         let mut count = 0u32; // we want to limit the number of times we perform this operation
//         let mut norm = T::infinity(); // so that we get at least one iteration
//         let mut r_mat = upper_mat; // represents rotation and scale

//         while count < 100 && norm > T::from(0.0001).unwrap() {
//             let r_next = match r_mat.transpose().inverse() {
//                 Some(mat) => mat.scale(T::from(0.5).unwrap()),
//                 _ => return None,
//             };

//             let n0 = (r_mat[0] - r_next[0]).abs().horizontal_add();
//             let n1 = (r_mat[1] - r_next[1]).abs().horizontal_add();
//             let n2 = (r_mat[2] - r_next[2]).abs().horizontal_add();
            
//             norm = n0.max(n1.max(n2));
//             r_mat = r_next;
//         }

//         let rot = Quat::from_mat4(r_mat);
//         let scale = match r_mat.inverse() {
//             Some(mat) => mat * upper_mat,
//             _ => return None,
//         };

//         Some((trans, rot, scale))
//     }
// }
