use crate::math::vector::Vec3;
use crate::math::numbers::Float;

/// The core ray structure:
#[derive(Clone, Copy, Debug)]
pub struct Ray<T: Float> {
    pub org: Vec3<T>,
    pub dir: Vec3<T>,
}

// Some simple functions that might be useful for rays:
impl<T: Float> Ray<T> {
    pub fn point_at(self, t: T) -> Vec3<T> {
        self.org + self.dir.scale(t)
    }
}

/// Differential component of a ray (not the ray itself, mind you)
#[derive(Clone, Copy, Debug)]
pub struct RayDiff<T: Float> {
    pub rx: Ray<T>,
    pub ry: Ray<T>,
}
