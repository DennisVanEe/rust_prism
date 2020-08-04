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
    /// The max extent of the ray to consider when tracing against geometry.
    pub t_far: T,
    /// Where along the ray to start checking for intersections.
    pub t_near: T,
}

impl<T: Float> Ray<T> {
    /// Constructs a new Ray for intersecting a scene, that is, without a parametric restriction.
    pub fn new(org: Vec3<T>, dir: Vec3<T>, time: T) -> Self {
        Ray {
            org,
            dir,
            time,
            t_far: T::infinity(),
            t_near: T::SELF_INT_COMP,
        }
    }

    /// Calculates a point along the ray given a parametric parameter.
    pub fn point_at(self, t: T) -> Vec3<T> {
        self.org + self.dir.scale(t)
    }
}

/// This ray represents a ray and ray_diff used by
/// the camera to genarate primary rays.
#[derive(Clone, Copy, Debug)]
pub struct PrimaryRay<T: Float> {
    pub ray: Ray<T>,
    pub ray_diff: RayDiff<T>,
}

impl<T: Float> PrimaryRay<T> {
    pub fn get_ray_dx(self) -> Ray<T> {
        Ray {
            org: self.ray_diff.rx_org,
            dir: self.ray_diff.rx_dir,
            time: self.ray.time,
            t_far: self.ray.t_far,
            t_near: self.ray.t_near,
        }
    }

    pub fn get_ray_dy(self) -> Ray<T> {
        Ray {
            org: self.ray_diff.ry_org,
            dir: self.ray_diff.ry_dir,
            time: self.ray.time,
            t_far: self.ray.t_far,
            t_near: self.ray.t_near,
        }
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
