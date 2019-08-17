use crate::math::vector::Vec3;
use crate::math::matrix::Mat4;

use num_traits::Float;

use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Quat<T: Copy + Float> {
    pub xyz: Vec3<T>,
    pub w: T,
}

impl<T: Copy + Float> Quat<T> {
    pub fn from_mat4(mat: Mat4) -> Self {

    }

    pub fn to_mat4(self) -> Mat4 {
        let x2 = self.xyz.x * self.xyz.x;
        let y2 = self.xyz.y * self.xyz.y;
        let z2 = self.xyz.z * self.xyz.z;

        let 
    }

    pub fn dot(self, o: Quat<T>) -> T {
        self.xyz.dot(o.xyz) + self.w * o.w
    }

    pub fn length2(self) -> T {
        self.dot(self)
    }

    pub fn length(self) -> T {
        self.length2().sqrt()
    }

    pub fn normalize(self) -> Self {
        let inv_len = T::one() / self.length();
        self.scale(inv_len)
    }

    pub fn scale(self, s: T) -> Self {
        Quat {
            xyz: self.xyz.scale(s),
            w: self.w * s,
        }
    }
}

impl<T: Copy + Float> Mul for Quat<T> {
    type Output = Self;

    fn mul(self, o: Quat<T>) -> Self {
        Quat {
            xyz: self.xyz.cross(o.xyz) + o.xyz.scale(self.w) + self.xyz.scale(o.w),
            w: self.w * o.w - self.xyz.dot(o.xyz),
        }
    }
}

impl<T: Copy + Float> Add for Quat<T> {
    type Output = Self;

    fn add(self, o: Quat<T>) -> Self {
        Quat {
            xyz: self.xyz + o.xyz,
            w: self.w + o.w,
        }
    }
}

impl<T: Copy + Float> Sub for Quat<T> {
    type Output = Self;

    fn sub(self, o: Quat<T>) -> Self {
        Quat {
            xyz: self.xyz - o.xyz,
            w: self.w - o.w,
        }
    }
}
