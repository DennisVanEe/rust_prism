use crate::math::vector::Vec3;

use num_traits::Float;

use std::cmp::PartialOrd;
use std::f64;

// This creates a coordinate system given only a single vector.
pub fn coord_system<T: Float>(v1: Vec3<T>) -> (Vec3<T>, Vec3<T>) {
    let v2 = if v1.x.abs() > v1.y.abs() {
        Vec3 {
            x: -v1.x,
            y: T::zero(),
            z: v1.x,
        }
    } else {
        Vec3 {
            x: T::zero(),
            y: v1.z,
            z: -v1.y,
        }
    }
    .normalize();

    let v3 = v1.cross(v2);

    (v2, v3)
}

// Aligns a vector vec so that it faces the same direction as the refv vector
// by negating or not negating it.
pub fn align<T: Float>(refv: Vec3<T>, vec: Vec3<T>) -> Vec3<T> {
    if refv.dot(vec) < T::zero() {
        -vec
    } else {
        vec
    }
}

// Used for handling errors:

pub fn gamma_f32(n: i32) -> f32 {
    let n = n as f32;
    let half_eps = std::f32::EPSILON / 2f32;
    (n * half_eps) / (1f32 - n * half_eps)
}

pub fn gamma_f64(n: i64) -> f64 {
    let n = n as f64;
    let half_eps = std::f64::EPSILON / 2.;
    (n * half_eps) / (1. - n * half_eps)
}

// This is used so that we can have efficient comparisons
// with PartialOrd types:

pub fn min<T: PartialOrd>(v0: T, v1: T) -> T {
    if v0 < v1 {
        v0
    } else {
        v1
    }
}

pub fn max<T: PartialOrd>(v0: T, v1: T) -> T {
    if v0 > v1 {
        v0
    } else {
        v1
    }
}

// Solves the quadratic equation robustly:
pub fn quadratic<T: Float>(a: T, b: T, c: T) -> Option<(T, T)> {
    let disc = b * b - T::from(4).unwrap() * a * c;
    if disc < T::zero() {
        return None;
    }
    let root_disc = disc.sqrt();

    let q = if b < T::zero() {
        -T::from(0.5).unwrap() * (b - root_disc)
    } else {
        -T::from(0.5).unwrap() * (b + root_disc)
    };

    let t0 = q / a;
    let t1 = c / q;

    Some((t0.min(t1), t0.max(t1)))
}
