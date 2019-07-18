use crate::math::vector::{Vec2, Vec3};
use crate::math::ray::Ray;

use num_traits::{Float, Signed, Zero};

use std::ops::{Add, Index, Mul, Neg, Sub};
use std::mem::swap;

pub struct BBox2<T: Copy> {
    pub pmin: Vec2<T>,
    pub pmax: Vec2<T>,
}

pub type BBox2f = BBox2<f32>;
pub type BBox2d = BBox2<f64>;
pub type BBox2i = BBox2<i32>;

pub struct BBox3<T: Copy> {
    pub pmin: Vec3<T>,
    pub pmax: Vec3<T>,
}

pub type BBox3f = BBox3<f32>;
pub type BBox3d = BBox3<f64>;
pub type BBox3i = BBox3<i32>;

impl<T: Float + Copy> BBox2<T> {
    pub fn new() -> Self {
        BBox2 {
            pmin: Vec2 {
                x: T::infinity(),
                y: T::infinity(),
            },
            pmax: Vec2 {
                x: -T::infinity(),
                y: -T::infinity(),
            }
        }
    }
}

impl<T: Copy> Index<usize> for BBox2<T> {
    type Output = Vec2<T>;

    fn index(&self, i: usize) -> &Vec2<T> {
        match i {
            0 => &self.pmin,
            1 => &self.pmax,
            _ => panic!("Index out of range for BBox2"),
        }
    }
}

impl<T: Copy> BBox3<T> {
    pub fn corner(&self, i: usize) -> Vec3<T> {
        let x = (*self)[i & 1].x;
        let y = (*self)[ if i & 2 != 0 { 1 } else { 0 } ].y;
        let z = (*self)[ if i & 4 != 0 { 1 } else { 0 } ].z;
        Vec3 { x, y, z }
    }
}

impl<T: Float + Copy> BBox3<T> {
    pub fn new() -> Self {
        BBox3 {
            pmin: Vec3 {
                x: T::infinity(),
                y: T::infinity(),
                z: T::infinity(),
            },
            pmax: Vec3 {
                x: -T::infinity(),
                y: -T::infinity(),
                z: -T::infinity(),
            }
        }
    }
}

impl BBox3<f32> {
    pub fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        let mut t0 = 0f32;
        let mut t1 = ray.max_time;

        for i in 0..3 {
            let inv_dir = 1f32 / ray.dir[i];
            let mut t_near = (self.pmin[i] - ray.org[i]) * inv_dir;
            let mut t_far = (self.pmax[i] - ray.org[i]) * inv_dir;    
            if t_near > t_far {
                swap(&mut t_near, &mut t_far);
            }

            t0 = if t_near > t0 { t_near } else { t0 };
            t1 = if t_far < t1 { t_far } else { t1 };

            if t0 > t1 {
                return None;
            }
        }

        Some((t0, t1))
    }
}

impl<T: Copy> Index<usize> for BBox3<T> {
    type Output = Vec3<T>;

    fn index(&self, i: usize) -> &Vec3<T> {
        match i {
            0 => &self.pmin,
            1 => &self.pmax,
            _ => panic!("Index out of range for BBox2"),
        }
    }
}
