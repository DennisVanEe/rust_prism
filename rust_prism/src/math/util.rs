use crate::math::numbers::Float;
use crate::math::vector::{Vec2, Vec3};

use std::cmp::PartialOrd;
use std::f64;

// Morton Encoding for 2D values:
pub fn morton_from_2d(xy: Vec2<u32>) -> u64 {
    // Only compile if we have support for bmi2:
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "bmi2"
    ))]
    {
        // Only perform this faster approach if we can:
        use core::arch::x86_64::_pdep_u64;
        unsafe {
            // What PDEP (parallel bits deposit) does here (with these masks) is space the bits so that there is a
            // 0 bit between each of the bits. We use different masks to pick different starting points. Once that
            // is done, we then combine them with a bitwise or:
            return _pdep_u64(xy.x as u64, 0x5555555555555555)
                | _pdep_u64(xy.y as u64, 0xaaaaaaaaaaaaaaa);
        }
    }

    // The fall back technique (so, non x86 machines, or no support for bmi2)
    // Our own little pdep function (that only works for the mask 0x55...):
    fn pdep(n: u64) -> u64 {
        let n = (n | (n << 16)) & 0x0000ffff0000ffff;
        let n = (n | (n << 8)) & 0x00ff00ff00ff00ff;
        let n = (n | (n << 4)) & 0x0f0f0f0f0f0f0f0f;
        let n = (n | (n << 2)) & 0x3333333333333333;
        let n = (n | (n << 1)) & 0x5555555555555555;
        n
    }

    // Then we can finally or the result:
    pdep(xy.x as u64) | pdep((xy.y as u64) << 1)
}

pub fn morton_to_2d(m: u64) -> Vec2<u32> {
    // Only compile if we have support for bmi2:
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "bmi2"
    ))]
    {
        // Only perform this faster approach if we can:
        use core::arch::x86_64::_pext_u64;

        unsafe {
            // Here, PEXT basically does the opposite of what we saw above and extracts the bits:
            return Vec2 {
                x: _pext_u64(m, 0x5555555555555555) as u32,
                y: _pext_u64(m, 0xaaaaaaaaaaaaaaaa) as u32,
            };
        }
    }

    // The fall back technique (so, non x86 machines, or no support for bmi2)
    // Our own little pdext function (that only works for the mask 0x55...):
    fn pext(n: u64) -> u64 {
        let n = n & 0x5555555555555555;
        let n = (n | (n >> 1)) & 0x3333333333333333;
        let n = (n | (n >> 2)) & 0x0f0f0f0f0f0f0f0f;
        let n = (n | (n >> 4)) & 0x00ff00ff00ff00ff;
        let n = (n | (n >> 8)) & 0x0000ffff0000ffff;
        let n = (n | (n >> 16)) & 0x00000000ffffffff;
        n
    }

    Vec2 {
        x: pext(m) as u32,
        y: pext(m >> 1) as u32,
    }
}

// Reverses the bits in a u32 number:
pub fn reverse_u32(n: u32) -> u32 {
    let n = (n << 16) | (n >> 16);
    let n = ((n & 0x00ff00ff) << 8) | ((n & 0xff00ff00) >> 8);
    let n = ((n & 0x0f0f0f0f) << 4) | ((n & 0xf0f0f0f0) >> 4);
    let n = ((n & 0x33333333) << 2) | ((n & 0xcccccccc) >> 2);
    let n = ((n & 0x55555555) << 1) | ((n & 0xaaaaaaaa) >> 1);
    n
}

// Reverses the bits in a u64 number:
pub fn reverse_u64(n: u64) -> u64 {
    let n0 = reverse_u32(n as u32) as u64;
    let n1 = reverse_u32((n >> 32) as u32) as u64;
    (n0 << 32) | n1
}

// Computes the grey code value for an unsigned 32 bit value:
pub fn greycode_u32(n: u32) -> u32 {
    (n >> 1) ^ n
}

// Computes the grey code value for an unsigned 64 bit value:
pub fn greycode_u64(n: u64) -> u64 {
    (n >> 1) ^ n
}

// Rounds up to the nearest power of 2 for u32 numbers:
pub fn next_pow2_u32(n: u32) -> u32 {
    // The idea is to essentially set it so that all bits are set
    // from least significant bit to most significant bit already set.
    // Then when we add 1 we would "roll over":

    // Decrement by 1 so that it maps to itself (doesn't just get the next power of 2):
    let n = n - 1;
    let n = n | n >> 1;
    let n = n | n >> 2;
    let n = n | n >> 4;
    let n = n | n >> 8;
    let n = n | n >> 16;
    n + 1
}

// Rounds up to the nearest power of 2 for u64 numbers:
pub fn next_pow2_u64(n: u64) -> u64 {
    // The idea is to essentially set it so that all bits are set
    // from least significant bit to most significant bit already set.
    // Then when we add 1 we would "roll over":
    let n = n - 1;
    let n = n | n >> 1;
    let n = n | n >> 2;
    let n = n | n >> 4;
    let n = n | n >> 8;
    let n = n | n >> 16;
    let n = n | n >> 32;
    n + 1
}

// This creates a coordinate system given only a single vector.
pub fn coord_system<T: Float>(v1: Vec3<T>) -> (Vec3<T>, Vec3<T>) {
    // v2 can easily be calculated by just negating one of the components:
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
    const HALF_EPS: f32 = std::f32::EPSILON / 2.;
    (n * HALF_EPS) / (1. - n * HALF_EPS)
}

pub fn gamma_f64(n: i64) -> f64 {
    let n = n as f64;
    const HALF_EPS: f64 = std::f64::EPSILON / 2.;
    (n * HALF_EPS) / (1. - n * HALF_EPS)
}

// This is used so that we can have efficient comparisons
// with PartialOrd types (like floats). According to the compiler
// explorer, this converts to the proper minsd/maxsd instruction:
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

// Solves the quadratic equation robustly.
// If no solution exists, Option is set:
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

// Reflect function:
pub fn reflect<T: Float>(wo: Vec3<T>, n: Vec3<T>) -> Vec3<T> {
    -wo + n.scale(T::two() * wo.dot(n))
}

// Refract function:
pub fn refract<T: Float>(wi: Vec3<T>, n: Vec3<T>, eta: T) -> Option<Vec3<T>> {
    let cos_theta_i = n.dot(wi);
    let sin2_theta_i = T::zero().max(T::one() - cos_theta_i * cos_theta_i);
    let sin2_theta_t = eta * eta * sin2_theta_i;
    if sin2_theta_t >= T::one() {
        return None;
    }
    let cos_theta_t = (T::one() - sin2_theta_t).sqrt();
    Some((-wi).scale(eta) + n.scale(cos_theta_i * eta - cos_theta_t))
}
