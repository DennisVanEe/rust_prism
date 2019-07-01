mod geometry;
mod math;

extern crate num_traits;

use math::matrix::Mat4;
use math::vector::Vec3;

fn main() {
    let dir = Vec3 {
        x: 4.5f32,
        y: 9.8f32,
        z: 1.7f32,
    };
    let trans = Mat4::translation(dir);
    let kk = trans.transpose();
    let trans2 = kk.transpose();

    println!("Hello, world!");
}
