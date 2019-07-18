mod geometry;
mod math;

use geometry::loader::ply;
use geometry::mesh::{calc_rayintinfo, Mesh};
use math::ray::Ray;
use math::vector::Vec3f;

fn main() {
    let mesh = ply::load_path("/home/dennis/Downloads/sphere.ply").unwrap();

    let org = Vec3f {
        x: -2f32,
        y: 0f32,
        z: 0f32,
    };
    let dir = Vec3f {
        x: -1f32,
        y: 0f32,
        z: 0f32,
    };
    let max_time = 100f32;
    let time = 1.2f32;
    let ray = Ray {
        org,
        dir,
        max_time,
        time,
    };
    let int_info = calc_rayintinfo(&ray);

    // Now let's try to intersect it:
    let num_tris = mesh.num_tri();
    for i in 0..num_tris {
        let triangle = mesh.get_tri(i);

        if triangle.intersect_test(&ray, &int_info, &mesh) {
            println!("intersection found!");
            break;
        }
    }

    println!("end of line has been reached");
}
