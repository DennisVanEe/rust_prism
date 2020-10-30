use crate::numbers::Float;
use crate::vector::Vec3;

/// A ray used to intersect a scene.
#[derive(Clone, Copy, Debug)]
pub struct Ray<T: Float> {
    /// The origin point of the ray.
    pub org: Vec3<T>,
    /// The direction vector of the ray.
    pub dir: Vec3<T>,
    /// The current time in the scene of the ray.
    pub time: T,
    /// The max extent of the ray.
    pub t_far: T,
    /// The min extent of the ray.
    pub t_near: T,
}

impl<T: Float> Ray<T> {
    /// Calculates a point along the ray given a parametric parameter.
    pub fn point_at(self, t: T) -> Vec3<T> {
        self.org + self.dir.scale(t)
    }
}
