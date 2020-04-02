// Clean this stuff up in the future...
// This is here just for now.

mod file_io;
mod geometry;
mod math;
mod memory;
//mod bvh;
//mod camera;
//mod film;
//mod filter;
//mod geometry;
//mod integrator;
//mod light;
//mod math;
//mod memory;
//mod sampler;
//mod scene;
//mod scene_loading;
//mod shading;
//mod spectrum;
//mod threading;
//mod transform;

use geometry::mesh::TriMesh;

fn main() {
    let result = file_io::ply::load_tri_mesh("E:\\apple.ply");
    let mesh = match result {
        Ok(m) => m,
        Err(msg) => panic!(msg),
    };

    println!("{}", mesh.has_nrm());
}
