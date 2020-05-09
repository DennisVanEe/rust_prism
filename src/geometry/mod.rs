pub mod bvh_object;
pub mod mesh;
pub mod ply;

use crate::math::ray::Ray;
use crate::math::vector::{Vec2, Vec3, Vec3Perm};

// Geometric interaction:
#[derive(Clone, Copy)]
pub struct Interaction {
    pub p: Vec3<f64>,  // intersection point
    pub n: Vec3<f64>,  // geometric normal (of triangle)
    pub wo: Vec3<f64>, // direction of intersection leaving the point

    pub t: f64, // the t value of the intersection of the ray

    pub uv: Vec2<f64>,   // uv coordinate at the intersection
    pub dpdu: Vec3<f64>, // vectors parallel to the triangle
    pub dpdv: Vec3<f64>,

    pub shading_n: Vec3<f64>,    // the shading normal at this point
    pub shading_dpdu: Vec3<f64>, // the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3<f64>,
    pub shading_dndu: Vec3<f64>, // the shading dndu, dndv at this point
    pub shading_dndv: Vec3<f64>,

    pub attribute_id: u32, // id of the material to use
    pub mesh_id: u32,      // id of the mesh where the intersection occured
}

#[derive(Clone, Copy)]
pub struct RayIntInfo {
    shear: Vec3<f64>,
    perm_dir: Vec3<f64>,
    perm: Vec3Perm,
}

// Given a ray, calculates the ray intersection information used for
// efficient ray-triangle intersection.
pub fn calc_rayintinfo(ray: Ray<f64>) -> RayIntInfo {
    let z = ray.dir.abs().max_dim();
    let x = if z == 2 { 0 } else { z + 1 };
    let y = if x == 2 { 0 } else { x + 1 };

    let perm = Vec3Perm::new(x, y, z);
    let perm_dir = ray.dir.permute(perm);

    let inv_perm_dir_z = 1. / perm_dir.z;
    let shear = Vec3 {
        x: -perm_dir.x * inv_perm_dir_z,
        y: -perm_dir.y * inv_perm_dir_z,
        z: inv_perm_dir_z,
    };

    RayIntInfo {
        shear,
        perm_dir,
        perm,
    }
}
