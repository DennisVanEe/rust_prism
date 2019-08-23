use crate::math::vector::Vec3;

use num_traits::Float;

/// The core ray structure:
#[derive(Clone, Copy, Debug)]
pub struct Ray<T: Float> {
    pub org: Vec3<T>,
    pub dir: Vec3<T>,
}

/// Differential component of a ray (not the ray itself, mind you)
#[derive(Clone, Copy, Debug)]
pub struct RayDiff<T: Float> {
    pub rx: Ray<T>,
    pub ry: Ray<T>,
}
