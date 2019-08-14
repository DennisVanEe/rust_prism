mod alloc;
mod geometry;
mod math;
mod util;

use geometry::loader::ply;
use geometry::mesh::{calc_rayintinfo, Mesh};
use geometry::mesh_bvh::MeshBVH;
use math::ray::Ray;
use math::vector::Vec3f;

fn main() {
    let mesh =
        ply::load_path("E:/Development/cpp_projects/prism/Prism/test_files/sphere.ply").unwrap();

    let org = Vec3f {
        x: 0f32,
        y: 0f32,
        z: 0f32,
    };
    let dir = Vec3f {
        x: -1f32,
        y: 1f32,
        z: -1f32,
    };
    let max_time = 100f32;
    let ray = Ray { org, dir };
    let int_info = calc_rayintinfo(ray);

    // Now let's try to intersect it:

    let bvh = MeshBVH::new(mesh, 32);
    if let Some(int) = bvh.intersect(ray, max_time, int_info) {
        println!("intersection found! {:?}", int.p);
    }

    println!("end of line has been reached");
}
