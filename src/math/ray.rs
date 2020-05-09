use crate::math::numbers::Float;
use crate::math::vector::Vec3;

/// A ray used to intersect a scene.
#[derive(Clone, Copy, Debug)]
pub struct Ray<T: Float> {
    /// The origin point of the ray.
    pub org: Vec3<T>,
    /// The direction vector of the ray.
    pub dir: Vec3<T>,
    /// The current time in the scene of the ray.
    pub time: T,
    /// The parametric extent of the ray. Usually only modified by intersection routines
    /// to allow for earlier termination.
    pub max_t: T,

    // The ray differential if present for this specific ray:
    pub ray_diff: Option<RayDiff<T>>,
}

impl<T: Float> Ray<T> {
    /// Constructs a new Ray for intersecting a scene, that is, without
    /// a parametric restriction.
    pub fn new(org: Vec3<T>, dir: Vec3<T>, time: T) -> Self {
        Ray {
            org,
            dir,
            time,
            max_t: T::infinity(),
            ray_diff: None,
        }
    }

    pub fn new_diff(org: Vec3<T>, dir: Vec3<T>, time: T, ray_diff: RayDiff<T>) -> Self {
        Ray {
            org,
            dir,
            time,
            max_t: T::infinity(),
            ray_diff: Some(ray_diff),
        }
    }

    /// Calculates a point along the ray given a parametric parameter.
    ///
    /// # Arguments
    /// * `t` - The parametric parameter.
    pub fn point_at(self, t: T) -> Vec3<T> {
        self.org + self.dir.scale(t)
    }
}

/// Differential component of a ray, that is, the ray slightly offset
/// in the x and the y direction.
#[derive(Clone, Copy, Debug)]
pub struct RayDiff<T: Float> {
    pub rx_org: Vec3<T>,
    pub rx_dir: Vec3<T>,

    pub ry_org: Vec3<T>,
    pub ry_dir: Vec3<T>,
}

impl<T: Float> RayDiff<T> {}
