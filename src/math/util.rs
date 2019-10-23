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
                | _pdep_u64(xy.y as u64, 0xAAAAAAAAAAAAAAAA);
        }
    }

    // The fall back technique (TODO: get something specifc for NEON and ARM based systems as well):

    // See: https://www.forceflow.be/2013/10/07/morton-encodingdecoding-through-bit-interleaving-implementations/
    // for details:
    let x = xy.x as u64;
    let y = xy.y as u64;

    let x = (x | (x << 16)) & 0x0000FFFF0000FFFF;
    let x = (x | (x << 8)) & 0x00FF00FF00FF00FF;
    let x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F;
    let x = (x | (x << 2)) & 0x3333333333333333;
    let x = (x | (x << 1)) & 0x5555555555555555;

    let y = (y | (y << 16)) & 0x0000FFFF0000FFFF;
    let y = (y | (y << 8)) & 0x00FF00FF00FF00FF;
    let y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F;
    let y = (y | (y << 2)) & 0x3333333333333333;
    let y = (y | (y << 1)) & 0x5555555555555555;

    x | (y << 1)
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

    // The fall back technique (TODO: get something specifc for NEON and ARM based systems as well):

    // See: https://www.forceflow.be/2013/10/07/morton-encodingdecoding-through-bit-interleaving-implementations/
    // for details:
    fn morton_1(x: u64) -> u32 {
        let x = x & 0x5555555555555555;
        let x = (x | (x >> 1)) & 0x3333333333333333;
        let x = (x | (x >> 2)) & 0x0F0F0F0F0F0F0F0F;
        let x = (x | (x >> 4)) & 0x00FF00FF00FF00FF;
        let x = (x | (x >> 8)) & 0x0000FFFF0000FFFF;
        let x = (x | (x >> 16)) & 0x00000000FFFFFFFF;
        x as u32
    }

    Vec2 {
        x: morton_1(m),
        y: morton_1(m >> 1),
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

pub fn reflect<T: Float>(wo: Vec3<T>, n: Vec3<T>) -> Vec3<T> {
    -wo + n.scale(T::two() * wo.dot(n))
}

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
