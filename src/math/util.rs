use crate::math::vector::Vec3;
use num_traits::Float;

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
