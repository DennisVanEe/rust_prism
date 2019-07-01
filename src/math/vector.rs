/// Defines a bunch of vector types and whatnot:
// Needs to be signed to support negation.
// Float is used to handle sqrt case and whatnot that may arise.
use num_traits::{Float, Signed};

use std::ops::{Add, Index, Mul, Neg, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T: Copy> {
    pub x: T,
    pub y: T,
}

pub type Vec2f = Vec2<f32>;
pub type Vec2i = Vec2<i32>;

#[derive(Copy, Clone, Debug)]
pub struct Vec3<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Vec3f = Vec3<f32>;
pub type Vec3d = Vec3<f64>;
pub type Vec3i = Vec3<i32>;

#[derive(Copy, Clone, Debug)]
pub struct Vec4<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub type Vec4f = Vec4<f32>;
pub type Vec4i = Vec4<i32>;

// Operations:

impl<T: Signed + Copy> Vec2<T> {
    pub fn abs(self) -> Vec2<T> {
        Vec2 {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> Vec2<T> {
    pub fn dot(self, o: Vec2<T>) -> T {
        self.x * o.x + self.y * o.y
    }

    pub fn scale(self, s: T) -> Vec2<T> {
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
}

// This is for operations that require a float (like a length function):
impl<T: Float> Vec2<T> {
    pub fn length(self) -> T {
        self.length2().sqrt()
    }

    pub fn normalize(self) -> Vec2<T> {
        let scale = T::one() / self.length();
        self.scale(scale)
    }
}

impl<T: Signed + Copy> Vec3<T> {
    pub fn abs(self) -> Vec3<T> {
        Vec3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> Vec3<T> {
    pub fn dot(self, o: Vec3<T>) -> T {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn scale(self, s: T) -> Vec3<T> {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    pub fn length2(self) -> T {
        self.dot(self)
    }
}

// Only supported for vec3:
impl<T: Mul<Output = T> + Sub<Output = T> + Copy> Vec3<T> {
    pub fn cross(self, o: Vec3<T>) -> Vec3<T> {
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

    pub fn normalize(self) -> Vec3<T> {
        let scale = T::one() / self.length();
        self.scale(scale)
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

    pub fn permute(self, perm: u32) -> Vec3<T> {
        match perm {
            8 /*xzy*/ => Vec3 { x: self.x, y: self.z, z: self.y },
            5 /*yzx*/ => Vec3 { x: self.y, y: self.z, z: self.x },
            9 /*yxz*/ => Vec3 { x: self.y, y: self.x, z: self.z },
            4 /*zyx*/ => Vec3 { x: self.z, y: self.y, z: self.x },
            6 /*zxy*/ => Vec3 { x: self.z, y: self.x, z: self.y },
            10 /*xyz*/ => Vec3 { x: self.x, y: self.y, z: self.z },
            _ => panic!("Invalid permutation value"),
        }
    }
}

impl<T: Signed + Copy> Vec4<T> {
    pub fn abs(self) -> Vec4<T> {
        Vec4 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
            w: self.w.abs(),
        }
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> Vec4<T> {
    pub fn dot(self, o: Vec4<T>) -> T {
        self.x * o.x + self.y * o.y + self.z * o.z + self.w * o.w
    }

    pub fn scale(self, s: T) -> Vec4<T> {
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

    pub fn normalize(self) -> Vec4<T> {
        let scale = T::one() / self.length();
        self.scale(scale)
    }
}

///////////////////////
// Trait Definitions //
///////////////////////

impl<T: Add<Output = T> + Copy> Add for Vec2<T> {
    type Output = Vec2<T>;

    fn add(self, o: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec2<T> {
    type Output = Vec2<T>;

    fn sub(self, o: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul for Vec2<T> {
    type Output = Vec2<T>;

    fn mul(self, o: Vec2<T>) -> Vec2<T> {
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

    fn neg(self) -> Vec2<T> {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec3<T> {
    type Output = Vec3<T>;

    fn add(self, o: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec3<T> {
    type Output = Vec3<T>;

    fn sub(self, o: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }
}

impl<T: Signed + Copy> Mul for Vec3<T> {
    type Output = Vec3<T>;

    fn mul(self, o: Vec3<T>) -> Vec3<T> {
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

impl<T: Neg<Output = T> + Copy> Neg for Vec3<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Vec3<T> {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec4<T> {
    type Output = Vec4<T>;

    fn add(self, o: Vec4<T>) -> Vec4<T> {
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

    fn sub(self, o: Vec4<T>) -> Vec4<T> {
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

    fn mul(self, o: Vec4<T>) -> Vec4<T> {
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

    fn neg(self) -> Vec4<T> {
        Vec4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}
