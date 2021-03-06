use crate::numbers::Float;
use crate::ray::Ray;
use crate::vector::{Vec2, Vec3};
use num_traits::Bounded;
use std::cmp::PartialOrd;
use std::ops::{Index, Sub};

/// A 2d bounding box.
#[derive(Clone, Copy, Debug)]
pub struct BBox2<T: PartialOrd + Bounded + Copy> {
    pub pmin: Vec2<T>,
    pub pmax: Vec2<T>,
}

impl<T: PartialOrd + Bounded + Copy> BBox2<T> {
    /// Creates a new `BBox2`.
    pub fn new() -> Self {
        BBox2 {
            pmin: Vec2 {
                x: T::max_value(),
                y: T::max_value(),
            },
            pmax: Vec2 {
                x: T::min_value(),
                y: T::min_value(),
            },
        }
    }

    /// Creates a new `BBox2` from two points.
    pub fn from_pnts(pnt0: Vec2<T>, pnt1: Vec2<T>) -> Self {
        BBox2 {
            pmin: pnt0.min(pnt1),
            pmax: pnt0.max(pnt1),
        }
    }

    /// Creates a new `BBox2` from a single point (box has no volume).
    pub fn from_pnt(pnt: Vec2<T>) -> Self {
        BBox2 {
            pmin: pnt,
            pmax: pnt,
        }
    }

    /// Creates a new `BBox2` that encompases the box and another point.
    pub fn combine_pnt(&self, pnt: Vec2<T>) -> Self {
        let pmin = self.pmin.min(pnt);
        let pmax = self.pmax.max(pnt);
        BBox2 { pmin, pmax }
    }

    /// Creates a new `BBox2` that encompases the box and another box.
    pub fn combine_bnd(&self, bnd: &BBox2<T>) -> Self {
        let pmin = self.pmin.min(bnd.pmin);
        let pmax = self.pmax.max(bnd.pmax);
        BBox2 { pmin, pmax }
    }
}

impl<T: PartialOrd + Bounded + Copy> Index<usize> for BBox2<T> {
    type Output = Vec2<T>;

    fn index(&self, i: usize) -> &Vec2<T> {
        match i {
            0 => &self.pmin,
            1 => &self.pmax,
            _ => panic!("Index out of range for BBox2"),
        }
    }
}

/// A 3d bounding box.
#[derive(Clone, Copy, Debug)]
pub struct BBox3<T: PartialOrd + Bounded + Copy> {
    pub pmin: Vec3<T>,
    pub pmax: Vec3<T>,
}

impl<T: PartialOrd + Bounded + Copy> BBox3<T> {
    /// Constructs a "initial" bounding box. An initial bounding box is one, when combined
    /// with another, takes on the value of that other bounding box.
    pub fn new_initial() -> Self {
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
            },
        }
    }

    /// Creates a new `BBox3` from two points.
    pub fn from_pnts(pnt0: Vec3<T>, pnt1: Vec3<T>) -> Self {
        BBox3 {
            pmin: pnt0.min(pnt1),
            pmax: pnt0.max(pnt1),
        }
    }

    /// Creates a new `BBox3` from a single point (box has no volume).
    pub fn from_pnt(pnt: Vec3<T>) -> Self {
        BBox3 {
            pmin: pnt,
            pmax: pnt,
        }
    }

    /// Used to get a specific corner given the index. Panics if the index is out of range.
    pub fn corner(self, i: usize) -> Vec3<T> {
        let x = self[i & 1].x;
        let y = self[if i & 2 != 0 { 1 } else { 0 }].y;
        let z = self[if i & 4 != 0 { 1 } else { 0 }].z;
        Vec3 { x, y, z }
    }

    /// Creates a new `BBox3` that encompases the box and another point.
    pub fn combine_pnt(self, pnt: Vec3<T>) -> Self {
        let pmin = self.pmin.min(pnt);
        let pmax = self.pmax.max(pnt);
        BBox3 { pmin, pmax }
    }

    /// Creates a new `BBox3` that encompases the box and another box.
    pub fn combine_bnd(self, bnd: BBox3<T>) -> Self {
        let pmin = self.pmin.min(bnd.pmin);
        let pmax = self.pmax.max(bnd.pmax);
        BBox3 { pmin, pmax }
    }
}

impl<T: Float + Bounded> BBox3<T> {
    /// Continious position of a point relative to the corners of the BBox.
    /// That is, if `pnt` is at `pmin`, then it's (0,0,0); if `pnt` is at `pmax`, then it's (1,1,1)
    pub fn offset(self, pnt: Vec3<T>) -> Vec3<T> {
        let o = pnt - self.pmin;
        Vec3 {
            x: if self.pmax.x > self.pmin.x {
                o.x / (self.pmax.x - self.pmin.x)
            } else {
                o.x
            },
            y: if self.pmax.y > self.pmin.y {
                o.y / (self.pmax.y - self.pmin.y)
            } else {
                o.y
            },
            z: if self.pmax.z > self.pmin.z {
                o.z / (self.pmax.z - self.pmin.z)
            } else {
                o.z
            },
        }
    }

    /// Returns the surface area of the bounding box.
    pub fn surface_area(self) -> T {
        let d = self.diagonal();
        T::two() * (d.x * d.y + d.x * d.y + d.y * d.z)
    }

    /// Returns the centroid of the bounding box.
    pub fn centroid(self) -> Vec3<T> {
        self.pmin.scale(T::half()) + self.pmax.scale(T::half())
    }

    // Performs an intersection of a bounding box, returning the t value along the ray
    // where the ray first hits the box and where it leaves the box.
    pub fn intersect(&self, ray: Ray<T>) -> Option<(T, T)> {
        let mut t0 = ray.t_near;
        let mut t1 = ray.t_far;

        for i in 0..3 {
            let inv_dir = T::one() / ray.dir[i];
            let mut t_near = (self.pmin[i] - ray.org[i]) * inv_dir;
            let mut t_far = (self.pmax[i] - ray.org[i]) * inv_dir;
            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
            }

            t0 = if t_near > t0 { t_near } else { t0 };
            t1 = if t_far < t1 { t_far } else { t1 };

            if t0 > t1 {
                return None;
            }
        }

        Some((t0, t1))
    }

    pub fn intersect_test(&self, ray: Ray<T>, inv_dir: Vec3<T>, is_dir_neg: Vec3<bool>) -> bool {
        // Use as indices:
        let is_dir_neg = [
            usize::from(is_dir_neg.x),
            usize::from(is_dir_neg.y),
            usize::from(is_dir_neg.z),
        ];

        let t_min = (self[is_dir_neg[0]].x - ray.org.x) * inv_dir.x;
        let t_max = (self[1 - is_dir_neg[0]].x - ray.org.x) * inv_dir.x;
        let ty_min = (self[is_dir_neg[1]].y - ray.org.y) * inv_dir.y;
        let ty_max = (self[1 - is_dir_neg[1]].y - ray.org.y) * inv_dir.y;

        // // Use this to take into account error connection
        // let t_max = t_max * (1. + 2. * gamma_f64(3));
        // let ty_max = ty_max * (1. + 2. * gamma_f64(3));

        if t_min > ty_max || ty_min > t_max {
            return false;
        }

        let t_min = if ty_min > t_min { ty_min } else { t_min };
        let t_max = if ty_max < t_max { ty_max } else { t_max };

        let tz_min = (self[is_dir_neg[2]].z - ray.org.z) * inv_dir.z;
        let tz_max = (self[1 - is_dir_neg[2]].z - ray.org.z) * inv_dir.z;

        // let tz_max = tz_max * (1. + 2. * gamma_f64(3));
        if t_min > tz_max || tz_min > t_max {
            return false;
        }

        let t_min = if tz_min > t_min { tz_min } else { t_min };
        let t_max = if tz_max < t_max { tz_max } else { t_max };

        t_min < ray.t_far && t_max > T::zero() && t_max > ray.t_near
    }
}

impl<T: Sub<Output = T> + PartialOrd + Bounded + Copy> BBox3<T> {
    /// Returns a vector spanning the diagonal of the bounding box.
    pub fn diagonal(self) -> Vec3<T> {
        self.pmax - self.pmin
    }

    /// Returns the index of the dimension with the highest extent.
    pub fn max_dim(self) -> usize {
        self.diagonal().max_dim()
    }
}

impl<T: PartialOrd + Bounded + Copy> Index<usize> for BBox3<T> {
    type Output = Vec3<T>;

    fn index(&self, i: usize) -> &Vec3<T> {
        match i {
            0 => &self.pmin,
            1 => &self.pmax,
            _ => panic!("Index out of range for BBox2"),
        }
    }
}
