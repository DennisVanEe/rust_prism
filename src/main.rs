mod geometry;
mod math;

use math::vector::Vec3f;
use geometry::mesh::Mesh;
use geometry::loader::ply;

fn main() {
    let mesh = ply::load_path("/home/dennis/Downloads/sphere.ply").unwrap();

    let v0 = unsafe { mesh.get_pos(0u32) };
    let v1 = unsafe { mesh.get_nrm(0u32) };

    let v2 = unsafe { mesh.get_pos(8u32) };
    let v3 = unsafe { mesh.get_nrm(8u32) };

    let v4 = unsafe { mesh.get_pos(mesh.num_vert() - 1) };
    let v5 = unsafe { mesh.get_nrm(mesh.num_vert() - 1) };


    println!("this is something {:?}, {:?}, {:?}, {:?}, {:?}, {:?}", v0, v1, v2, v3, v4, v5);
}
