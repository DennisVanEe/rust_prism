/// Defines a bunch of vector types and whatnot:
// Needs to be signed to support negation.
// Float is used to handle sqrt case and whatnot that may arise.
use crate::math::util::{max, min};

use num_traits::{Float, Signed, Zero};

use std::cmp::PartialOrd;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T: Copy> {
    pub x: T,
    pub y: T,
}

pub type Vec2f = Vec2<f32>;
pub type Vec2i = Vec2<i32>;

impl<T: Signed + Copy> Vec2<T> {
    pub fn abs(self) -> Self {
        Vec2 {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}

impl<T: Zero + Copy> Vec2<T> {
    pub fn zero() -> Self {
        Vec2 {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> Vec2<T> {
    pub fn dot(self, o: Vec2<T>) -> T {
        self.x * o.x + self.y * o.y
    }

    pub fn scale(self, s: T) -> Self {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }

    pub fn length2(self) -> T {
        self.dot(self)
    }
}

impl<T: PartialOrd + Copy> Vec2<T> {
    pub fn max_dim(self) -> usize {
        if self.x > self.y {
            0
        } else {
            1
        }
    }

    // Returns the maximum elements of the vector:
    pub fn max(self, o: Vec2<T>) -> Self {
        Vec2 {
            x: max(self.x, o.x),
            y: max(self.y, o.y),
        }
    }

    pub fn min(self, o: Vec2<T>) -> Self {
        Vec2 {
            x: min(self.x, o.x),
            y: min(self.y, o.y),
        }
    }
}

// This is for operations that require a float (like a length function):
impl<T: Float + Copy> Vec2<T> {
    pub fn length(self) -> T {
        self.length2().sqrt()
    }

    pub fn normalize(self) -> Self {
        let scale = T::one() / self.length();
        self.scale(scale)
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec2<T> {
    type Output = Vec2<T>;

    fn add(self, o: Vec2<T>) -> Self {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec2<T> {
    type Output = Vec2<T>;

    fn sub(self, o: Vec2<T>) -> Self {
        Vec2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul for Vec2<T> {
    type Output = Vec2<T>;

    fn mul(self, o: Vec2<T>) -> Self {
        Vec2 {
            x: self.x * o.x,
            y: self.y * o.y,
        }
    }
}

impl<T: Copy> Index<usize> for Vec2<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of range for Vec"),
        }
    }
}

impl<T: Neg<Output = T> + Copy> Neg for Vec2<T> {
    type Output = Vec2<T>;

    fn neg(self) -> Self {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Copy, Clone)]
pub enum Vec3Perm {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

pub type Vec3f = Vec3<f32>;
pub type Vec3d = Vec3<f64>;
pub type Vec3i = Vec3<i32>;

impl<T: Copy> Vec3<T> {
    pub fn from_vec4(v: Vec4<T>) -> Self {
        Vec3 {
            x: v.x, 
            y: v.y,
            z: v.z,
        }
    }

    pub fn permute(self, perm: Vec3Perm) -> Self {
        match perm {
            Vec3Perm::XYZ => Vec3 {
                x: self.x,
                y: self.y,
                z: self.z,
            },
            Vec3Perm::XZY => Vec3 {
                x: self.x,
                y: self.z,
                z: self.y,
            },
            Vec3Perm::YXZ => Vec3 {
                x: self.y,
                y: self.x,
                z: self.z,
            },
            Vec3Perm::YZX => Vec3 {
                x: self.y,
                y: self.z,
                z: self.x,
            },
            Vec3Perm::ZXY => Vec3 {
                x: self.z,
                y: self.x,
                z: self.y,
            },
            Vec3Perm::ZYX => Vec3 {
                x: self.z,
                y: self.y,
                z: self.x,
            },
        }
    }
}

impl<T: Signed + Copy> Vec3<T> {
    pub fn abs(self) -> Self {
        Vec3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    // Returns a vec of bools indicating whether or
    // not the entry is positive or negative:
    pub fn comp_wise_is_neg(self) -> Vec3<bool> {
        Vec3 {
            x: self.x.is_negative(),
            y: self.y.is_negative(),
            z: self.z.is_negative(),
        }
    }

    pub fn comp_wise_is_pos(self) -> Vec3<bool> {
        Vec3 {
            x: self.x.is_positive(),
            y: self.y.is_positive(),
            z: self.z.is_positive(),
        }
    }
}

impl<T: Zero + Copy> Vec3<T> {
    pub fn zero() -> Self {
        Vec3 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }
}

impl<T: Add<Output = T> + Copy> Vec3<T> {
    pub fn horizontal_add(self) -> T {
        self.x + self.y + self.z
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> Vec3<T> {
    pub fn dot(self, o: Vec3<T>) -> T {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn length2(self) -> T {
        self.dot(self)
    }
}

impl<T: Mul<Output = T> + Copy> Vec3<T> {
    // Scales the components by a scalar:
    pub fn scale(self, s: T) -> Self {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

impl<T: Div<Output = T> + Copy> Vec3<T> {
    // The inverse of scaling (s / vec):
    pub fn inv_scale(self, s: T) -> Self {
        Vec3 {
            x: s / self.x,
            y: s / self.y,
            z: s / self.z,
        }
    }
}

// Only supported for vec3:
impl<T: Mul<Output = T> + Sub<Output = T> + Copy> Vec3<T> {
    pub fn cross(self, o: Vec3<T>) -> Self {
        let x = self.y * o.z - self.z * o.y;
        let y = self.z * o.x - self.x * o.z;
        let z = self.x * o.y - self.y * o.x;
        Vec3 { x, y, z }
    }
}

impl<T: Float> Vec3<T> {
    pub fn length(self) -> T {
        self.length2().sqrt()
    }

    pub fn normalize(self) -> Self {
        let scale = T::one() / self.length();
        self.scale(scale)
    }

    pub fn lerp(self, v1: Self, time: T) -> Self {
        self.scale(T::one() - time) + v1.scale(time)
    }
}

impl<T: PartialOrd + Copy> Vec3<T> {
    pub fn max_dim(self) -> usize {
        if self.x > self.y && self.x > self.z {
            0
        } else if self.y > self.z {
            1
        } else {
            2
        }
    }

    pub fn min(self, o: Self) -> Self {
        Vec3 {
            x: min(self.x, o.x),
            y: min(self.y, o.y),
            z: min(self.z, o.z),
        }
    }

    pub fn max(self, o: Self) -> Self {
        Vec3 {
            x: max(self.x, o.x),
            y: max(self.y, o.y),
            z: max(self.z, o.z),
        }
    }
}

impl Vec3Perm {
    // Given a permutation, convert it to the proper enum:
    pub fn new(x: usize, y: usize, z: usize) -> Vec3Perm {
        let perm_code = x + 2 * y + 4 * z;
        match perm_code {
            8 /*xzy*/ => Vec3Perm::XZY,
            5 /*yzx*/ => Vec3Perm::YZX,
            9 /*yxz*/ => Vec3Perm::YXZ,
            4 /*zyx*/ => Vec3Perm::ZYX,
            6 /*zxy*/ => Vec3Perm::ZXY,
            10 /*xyz*/ => Vec3Perm::XYZ,
            // TODO: support more permutations:
            _ => panic!("Invalid permutation number for Vec3"),
        }
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec3<T> {
    type Output = Vec3<T>;

    fn add(self, o: Vec3<T>) -> Self {
        Vec3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec3<T> {
    type Output = Vec3<T>;

    fn sub(self, o: Vec3<T>) -> Self {
        Vec3 {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul for Vec3<T> {
    type Output = Vec3<T>;

    fn mul(self, o: Vec3<T>) -> Self {
        Vec3 {
            x: self.x * o.x,
            y: self.y * o.y,
            z: self.z * o.z,
        }
    }
}

impl<T: Copy> Index<usize> for Vec3<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of range for Vec"),
        }
    }
}

impl<T: Copy> IndexMut<usize> for Vec3<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of range for Vec"),
        }
    }
}

impl<T: Neg<Output = T> + Copy> Neg for Vec3<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Self {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vec4<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub type Vec4f = Vec4<f32>;
pub type Vec4i = Vec4<i32>;

impl<T: Copy> Vec4<T> {
    pub fn from_vec3(v: Vec3<T>, w: T) -> Self {
        Vec4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w
        }
    }
}

impl<T: Signed + Copy> Vec4<T> {
    pub fn abs(self) -> Self {
        Vec4 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
            w: self.w.abs(),
        }
    }
}

impl<T: Zero + Copy> Vec4<T> {
    pub fn zero() -> Self {
        Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
            w: T::zero(),
        }
    }
}

impl<T: Add<Output = T> + Copy> Vec4<T> {
    pub fn horizontal_add(self) -> T {
        self.x + self.y + self.z + self.w
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> Vec4<T> {
    pub fn dot(self, o: Vec4<T>) -> T {
        self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w
    }

    pub fn scale(self, s: T) -> Self {
        Vec4 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
            w: self.w * s,
        }
    }

    pub fn length2(self) -> T {
        self.dot(self)
    }
}

impl<T: Float> Vec4<T> {
    pub fn length(&self) -> T {
        self.length2().sqrt()
    }

    pub fn normalize(self) -> Self {
        let scale = T::one() / self.length();
        self.scale(scale)
    }

    pub fn lerp(self, v1: Self, time: T) -> Self {
        self.scale(T::one() - time) + v1.scale(time)
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec4<T> {
    type Output = Vec4<T>;

    fn add(self, o: Vec4<T>) -> Self {
        Vec4 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
            w: self.w + o.w,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec4<T> {
    type Output = Vec4<T>;

    fn sub(self, o: Vec4<T>) -> Self {
        Vec4 {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
            w: self.w - o.w,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul for Vec4<T> {
    type Output = Vec4<T>;

    fn mul(self, o: Vec4<T>) -> Self {
        Vec4 {
            x: self.x * o.x,
            y: self.y * o.y,
            z: self.z * o.z,
            w: self.w * o.w,
        }
    }
}

impl<T: Copy> Index<usize> for Vec4<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Index out of range for Vec"),
        }
    }
}

impl<T: Neg<Output = T> + Copy> Neg for Vec4<T> {
    type Output = Vec4<T>;

    fn neg(self) -> Self {
        Vec4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}
