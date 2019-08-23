// A n animated transform is one that animates between
// two static transformations.

use crate::transform::Transform;
use crate::transform::static_transform::StaticTransform;
use crate::math::matrix::{Mat3, Mat4};
use crate::math::vector::{Vec3, Vec4};
use crate::math::ray::Ray;
use crate::math::bbox::BBox3;

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