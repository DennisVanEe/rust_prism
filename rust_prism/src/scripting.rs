//! A module for handling the scripting system used to load scene files.
//!
//! Scene files are described in Rust using the Rhai scripting language. This allows for more procedural scene
//! descriptions or more straight forward scene descriptions depending on the situation.

use crate::transform::Transf;
use array_init::array_init;
use pmath::matrix::Mat3x4;
use pmath::vector::{Vec2, Vec3, Vec4};
use rhai::{Array, Engine};

/// Given a scripting Engine, registers the math types.
pub fn register_math_types(engine: &mut Engine) {
    register_vectors(engine);
    regiser_transf(engine);
}

pub fn regiser_transf(engine: &mut Engine) {
    engine
        .register_type::<Transf>()
        .register_type_with_name::<Transf>("Transf")
        .register_fn("new_transf", |arr: &mut Array| {
            // The array must be of size 12 and is in row-major order (read left to right, downward).
            // This will "throw" if this isn't gauranteed to happen:
            let native_arr: [f64; 12] = array_init(|i| arr[i].as_float().unwrap() as f64);
            let mat = Mat3x4::from_arr(native_arr);
            Transf::from_mat3x4(mat)
        })
        .register_fn("new_identity", || Transf::new_identity())
        .register_fn("new_translate", |trans: &mut Vec3<f64>| {
            Transf::new_translate(*trans)
        })
        .register_fn("new_scale", |scale: &mut Vec3<f64>| {
            Transf::new_scale(*scale)
        })
        .register_fn("new_rotate", |deg: f64, axis: &mut Vec3<f64>| {
            Transf::new_rotate(deg, *axis)
        })
        .register_fn("*", |t1: &mut Transf, t2: &mut Tranf| t1 * t2)
        .register_indexer_get(|v: &mut Transf, index: i64| v.get_frd()[index as usize]);
}

pub fn register_vectors(engine: &mut Engine) {
    // Register a vec2:
    engine
        .register_type::<Vec2<f64>>()
        .register_type_with_name::<Vec2<f64>>("Vec2")
        .register_fn("new_vec2", |x: f64, y: f64| Vec2 { x, y })
        .register_fn("+", |v1: &mut Vec2<f64>, v2: &mut Vec2<f64>| v1 + v2)
        .register_fn("-", |v1: &mut Vec2<f64>, v2: &mut Vec2<f64>| v1 - v2)
        .register_fn("*", |v1: &mut Vec2<f64>, v2: &mut Vec2<f64>| v1 * v2)
        .register_fn("/", |v1: &mut Vec2<f64>, v2: &mut Vec2<f64>| v1 / v2)
        .register_fn("scale", |v: &mut Vec2<f64>, s: f64| v.scale(s))
        .register_get_set(
            "x",
            |v: &mut Vec2<f64>| v.x,
            |v: &mut Vec2<f64>, s: f64| {
                v.x = s;
            },
        )
        .register_get_set(
            "y",
            |v: &mut Vec2<f64>| v.y,
            |v: &mut Vec2<f64>, s: f64| {
                v.y = s;
            },
        )
        .register_indexer_get_set(
            |v: &mut Vec2<f64>, index: i64| v[index as usize],
            |v: &mut Vec2<f64>, index: i64, s: f64| {
                v[index as usize] = s;
            },
        );

    // Register a vec3:
    engine
        .register_type::<Vec3<f64>>()
        .register_type_with_name::<Vec3<f64>>("Vec3")
        .register_fn("new_vec3", |x: f64, y: f64, z: f64| Vec3 { x, y, z })
        .register_fn("+", |v1: &mut Vec3<f64>, v2: &mut Vec3<f64>| v1 + v2)
        .register_fn("-", |v1: &mut Vec3<f64>, v2: &mut Vec3<f64>| v1 - v2)
        .register_fn("*", |v1: &mut Vec3<f64>, v2: &mut Vec3<f64>| v1 * v2)
        .register_fn("/", |v1: &mut Vec3<f64>, v2: &mut Vec3<f64>| v1 / v2)
        .register_fn("scale", |v: &mut Vec3<f64>, s: f64| v.scale(s))
        .register_get_set(
            "x",
            |v: &mut Vec3<f64>| v.x,
            |v: &mut Vec3<f64>, s: f64| {
                v.x = s;
            },
        )
        .register_get_set(
            "y",
            |v: &mut Vec3<f64>| v.y,
            |v: &mut Vec3<f64>, s: f64| {
                v.y = s;
            },
        )
        .register_get_set(
            "z",
            |v: &mut Vec3<f64>| v.z,
            |v: &mut Vec3<f64>, s: f64| {
                v.z = s;
            },
        )
        .register_indexer_get_set(
            |v: &mut Vec3<f64>, index: i64| v[index as usize],
            |v: &mut Vec3<f64>, index: i64, s: f64| {
                v[index as usize] = s;
            },
        );

    // Register a vec4:
    engine
        .register_type::<Vec4<f64>>()
        .register_type_with_name::<Vec4<f64>>("Vec2")
        .register_fn("new_vec4", |x: f64, y: f64, z: f64, w: f64| Vec4 {
            x,
            y,
            z,
            w,
        })
        .register_fn("+", |v1: &mut Vec4<f64>, v2: &mut Vec4<f64>| v1 + v2)
        .register_fn("-", |v1: &mut Vec4<f64>, v2: &mut Vec4<f64>| v1 - v2)
        .register_fn("*", |v1: &mut Vec4<f64>, v2: &mut Vec4<f64>| v1 * v2)
        .register_fn("/", |v1: &mut Vec4<f64>, v2: &mut Vec4<f64>| v1 / v2)
        .register_fn("scale", |v: &mut Vec4<f64>, s: f64| v.scale(s))
        .register_get_set(
            "x",
            |v: &mut Vec4<f64>| v.x,
            |v: &mut Vec4<f64>, s: f64| {
                v.x = s;
            },
        )
        .register_get_set(
            "y",
            |v: &mut Vec4<f64>| v.y,
            |v: &mut Vec4<f64>, s: f64| {
                v.y = s;
            },
        )
        .register_get_set(
            "z",
            |v: &mut Vec4<f64>| v.z,
            |v: &mut Vec4<f64>, s: f64| {
                v.z = s;
            },
        )
        .register_get_set(
            "w",
            |v: &mut Vec4<f64>| v.w,
            |v: &mut Vec4<f64>, s: f64| {
                v.w = s;
            },
        )
        .register_indexer_get_set(
            |v: &mut Vec4<f64>, index: i64| v[index as usize],
            |v: &mut Vec4<f64>, index: i64, s: f64| {
                v[index as usize] = s;
            },
        );
}
