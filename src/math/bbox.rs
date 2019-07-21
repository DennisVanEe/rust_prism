use crate::math::vector::{Vec2, Vec3, Vec3f};
use crate::math::ray::Ray;
use crate::math::util::gamma_f32;

use num_traits::{Bounded, Float, Signed, Zero};

use std::ops::{Add, Index, Mul, Neg, Sub};
use std::mem::swap;

pub struct BBox2<T: Bounded + Copy> {
    pub pmin: Vec2<T>,
    pub pmax: Vec2<T>,
}

pub type BBox2f = BBox2<f32>;
pub type BBox2d = BBox2<f64>;
pub type BBox2i = BBox2<i32>;

pub struct BBox3<T: Bounded + Copy> {
    pub pmin: Vec3<T>,
    pub pmax: Vec3<T>,
}

pub type BBox3f = BBox3<f32>;
pub type BBox3d = BBox3<f64>;
pub type BBox3i = BBox3<i32>;

//
// --------------------------------------
//

impl<T: Bounded + Copy> BBox2<T> {
    pub fn new() -> Self {
        BBox2 {
            pmin: Vec2 {
                x: T::max_value(),
                y: T::max_value(),
            },
            pmax: Vec2 {
                x: T::min_value(),
                y: T::min_value(),
            }
        }
    }
}

// Floats have different min and max implementations:
impl<T: Float + Bounded + Copy> BBox2<T> {
    pub fn combine_pnt(&self, pnt: Vec2<T>) -> Self {
        let pmin = self.pmin.min(pnt);
        let pmax = self.pmax.max(pnt);
        BBox2 { pmin, pmax }
    }

    pub fn combine_bnd(&self, bnd: BBox2<T>) -> Self {
        let pmin = self.pmin.min(bnd.pmin);
        let pmax = self.pmax.max(bnd.pmax);
        BBox2 { pmin, pmax }
    }
}

impl<T: Ord + Bounded + Copy> BBox2<T> {
    pub fn combine_pnt(&self, pnt: Vec2<T>) -> Self {
        let pmin = self.pmin.min(pnt);
        let pmax = self.pmax.max(pnt);
        BBox2 { pmin, pmax }
    }

    pub fn combine_bnd(&self, bnd: BBox2<T>) -> Self {
        let pmin = self.pmin.min(bnd.pmin);
        let pmax = self.pmax.max(bnd.pmax);
        BBox2 { pmin, pmax }
    }
}

impl<T: Bounded + Copy> Index<usize> for BBox2<T> {
    type Output = Vec2<T>;

    fn index(&self, i: usize) -> &Vec2<T> {
        match i {
            0 => &self.pmin,
            1 => &self.pmax,
            _ => panic!("Index out of range for BBox2"),
        }
    }
}

//
// --------------------------------------
//

impl<T: Bounded + Copy> BBox3<T> {
    pub fn new() -> Self {
        BBox3 {
            pmin: Vec3 {
                x: T::max_value(),
                y: T::max_value(),
                z: T::max_value(),
            },
            pmax: Vec3 {
                x: T::min_value(),
                y: T::min_value(),
                z: T::min_value(),
            }
        }
    }

    pub fn corner(&self, i: usize) -> Vec3<T> {
        let x = (*self)[i & 1].x;
        let y = (*self)[ if i & 2 != 0 { 1 } else { 0 } ].y;
        let z = (*self)[ if i & 4 != 0 { 1 } else { 0 } ].z;
        Vec3 { x, y, z }
    }
}

// Floats have different min and max implementations:
impl<T: Float + Bounded + Copy> BBox3<T> {
    pub fn combine_pnt(&self, pnt: Vec3<T>) -> Self {
        let pmin = self.pmin.min(pnt);
        let pmax = self.pmax.max(pnt);
        BBox3 { pmin, pmax }
    }

    pub fn combine_bnd(&self, bnd: BBox3<T>) -> Self {
        let pmin = self.pmin.min(bnd.pmin);
        let pmax = self.pmax.max(bnd.pmax);
        BBox3 { pmin, pmax }
    }
}

impl<T: Ord + Bounded + Copy> BBox3<T> {
    pub fn combine_pnt(&self, pnt: Vec3<T>) -> Self {
        let pmin = self.pmin.min(pnt);
        let pmax = self.pmax.max(pnt);
        BBox3 { pmin, pmax }
    }

    pub fn combine_bnd(&self, bnd: BBox3<T>) -> Self {
        let pmin = self.pmin.min(bnd.pmin);
        let pmax = self.pmax.max(bnd.pmax);
        BBox3 { pmin, pmax }
    }
}

// Because intersections are always done with f32, we only implement the intersection
// algorithm for BBox3s that are made up of floats:
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

    pub fn intersect_test(&self, ray: &Ray, inv_dir: Vec3f, is_dir_neg: Vec3<bool>) -> bool {
        // Use as indices:
        let i_dir_neg = [ usize::from(is_dir_neg.x), usize::from(is_dir_neg.y), usize::from(is_dir_neg.z) ];

        let t_min = (self[i_dir_neg[0]].x - ray.org.x) * inv_dir.x;
        let t_max = (self[1 - i_dir_neg[0]].x - ray.org.x) * inv_dir.x;
        let ty_min = (self[i_dir_neg[1]].y - ray.org.y) * inv_dir.y;
        let ty_max = (self[1 - i_dir_neg[1]].y - ray.org.y) * inv_dir.y;

        // Use this to take into account error connection:
        let t_max = t_max * (1f32 + 2f32 * gamma_f32(3));
        let ty_max = ty_max * (1f32 + 2f32 * gamma_f32(3));

        if t_min > ty_max || ty_min > t_max {
            return false;
        }

        let t_min = if ty_min > t_min { ty_min } else { t_min };
        let t_max = if ty_max < t_max { ty_max } else { t_max };

        let tz_min = (self[i_dir_neg[2]].z - ray.org.z) * inv_dir.z;
        let tz_max = (self[1 - i_dir_neg[2]].z - ray.org.z) * inv_dir.z;

        let tz_max = tz_max * (1f32 + 2f32 * gamma_f32(3));
        if t_min > tz_max || tz_min > t_max {
            return false;
        }

        let t_min = if tz_min > t_min { tz_min } else { t_min };
        let t_max = if tz_max < t_max { tz_max } else { t_max };

        t_min < ray.max_time && t_max > 0f32
    }
}

impl<T: Bounded + Copy> Index<usize> for BBox3<T> {
    type Output = Vec3<T>;

    fn index(&self, i: usize) -> &Vec3<T> {
        match i {
            0 => &self.pmin,
            1 => &self.pmax,
            _ => panic!("Index out of range for BBox2"),
        }
    }
}
