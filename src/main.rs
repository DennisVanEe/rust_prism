mod geometry;
mod math;
mod memory;
mod scene_loading;
mod transform;

use geometry::loader::ply;
use geometry::mesh::{calc_rayintinfo, Mesh};
use geometry::mesh_bvh::MeshBVH;
use math::ray::Ray;
use math::vector::Vec3;

use std::time::{Duration, Instant};

fn main() {
    // let mesh =
    //     ply::load_path("E:/Development/cpp_projects/prism/Prism/test_files/sphere.ply").unwrap();

    // let org = Vec3f {
    //     x: 0f32,
    //     y: 0f32,
    //     z: 0f32,
    // };
    // let dir = Vec3f {
    //     x: -1f32,
    //     y: 1f32,
    //     z: -1f32,
    // };
    // let max_time = 100f32;
    // let ray = Ray { org, dir };
    // let int_info = calc_rayintinfo(ray);

    // // Now let's try to intersect it:

    // let now = Instant::now();
    // let bvh = MeshBVH::new(mesh, 32);
    // let later = now.elapsed();

    // let now2 = Instant::now();
    // let result = bvh.intersect(ray, max_time, int_info);
    // let later2 = now2.elapsed();

    // if let Some(int) = result {
    //     println!("intersection found!");
    // }

    // // 138840

    // println!(
    //     "construction: {}, intersection: {}",
    //     later.as_micros(),
    //     later2.as_micros()
    // );
}
