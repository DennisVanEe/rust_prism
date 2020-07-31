use crate::math::numbers::Float;
use crate::math::vector::{Vec2, Vec3};

/// Used for multiple-importance sampling. Produces a weight for the "f" part of the distrubtion.
pub fn balance_heuristic<T: Float>(num_f: u32, pdf_f: T, num_g: u32, pdf_g: T) -> T {
    let num_f = T::from(num_f).unwrap();
    let num_g = T::from(num_g).unwrap();
    (num_f * pdf_f) / (num_f * pdf_f + num_g * pdf_g)
}

/// Used for multiple-importance sampling. Produces a weight for the "f" part of the distrubtion.
pub fn power_heuristic<T: Float>(num_f: u32, pdf_f: T, num_g: u32, pdf_g: T) -> T {
    let num_f = T::from(num_f).unwrap();
    let num_g = T::from(num_g).unwrap();
    let f = num_f * pdf_f;
    let g = num_g * pdf_g;
    (f * f) / (f * f + g * g)
}

pub fn uniform_sample_hemisphere<T: Float>(u: Vec2<T>) -> Vec3<T> {
    let z = u.x;
    let r = T::zero().max(T::one() - z * z).sqrt();
    let phi = T::two() * T::PI * u.y;
    Vec3 {
        x: r * phi.cos(),
        y: r * phi.sin(),
        z,
    }
}

pub fn uniform_hemisphere_pdf<T: Float>() -> T {
    T::INV_2PI
}

pub fn uniform_sample_sphere<T: Float>(u: Vec2<T>) -> Vec3<T> {
    let z = T::one() - T::two() * u.x;
    let r = T::zero().max(T::one() - z * z).sqrt();
    let phi = T::two() * T::PI * u.y;
    Vec3 {
        x: r * phi.cos(),
        y: r * phi.sin(),
        z,
    }
}

pub fn uniform_sphere_pdf<T: Float>() -> T {
    T::INV_4PI
}

pub fn concentric_sample_disk<T: Float>(u: Vec2<T>) -> Vec2<T> {
    // Map to [-1, 1]:
    let u_offset = u.scale(T::two()) - Vec2::one();
    if u_offset == Vec2::zero() {
        return Vec2::zero();
    }

    let (r, theta) = if u_offset.x.abs() > u_offset.y.abs() {
        (u_offset.x, T::PI_OVER_4 * (u_offset.y / u_offset.x))
    } else {
        (
            u_offset.y,
            T::PI_OVER_2 - T::PI_OVER_4 * (u_offset.x / u_offset.y),
        )
    };

    Vec2 {
        x: r * theta.cos(),
        y: r * theta.sin(),
    }
}

pub fn cos_sample_hemisphere<T: Float>(u: Vec2<T>) -> Vec3<T> {
    let d = concentric_sample_disk(u);
    let z = T::zero().max(T::one() - d.x * d.x - d.y * d.y).sqrt();
    Vec3 { x: d.x, y: d.y, z }
}

pub fn cos_sphere_pdf<T: Float>(cos_theta: T) -> T {
    cos_theta * T::INV_PI
}
