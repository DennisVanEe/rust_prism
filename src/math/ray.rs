use std::default::Default;

use super::vector::Vec3f;

/// The core ray structure:
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub org: Vec3f,
    pub dir: Vec3f,
    //pub time: f32,
    //pub max_time: f32,
}

/// Differential component of a ray (not the ray itself, mind you)
#[derive(Clone, Copy, Debug)]
pub struct RayDiff {
    pub rx: Ray,
    pub ry: Ray,
}

impl Default for Ray {
    fn default() -> Ray {
        Ray {
            org: Vec3f {
                x: 0f32,
                y: 0f32,
                z: 0f32,
            },
            dir: Vec3f {
                x: 0f32,
                y: 0f32,
                z: 0f32,
            },
            // time: 0f32,
            // max_time: std::f32::INFINITY,
        }
    }
}
