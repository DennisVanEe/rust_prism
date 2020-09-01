// Clean this stuff up in the future...
// This is here just for now.

#![allow(dead_code)]

mod camera;
mod fileio;
mod film;
mod filter;
mod geometry;
//mod integrator;
mod light;
mod sampler;
mod scene;
mod shading;
mod spectrum;
mod threading;
mod transform;

use camera::perspective::PerspectiveCamera;
use geometry::Geometry;
use integrator::normal::{NormalIntegrator, NormalIntegratorManager};
use pmath::vector::{Vec2, Vec3};
use transform::Transf;

use pmj;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use std::time::Instant;

const MODEL: &'static str = "/home/dennis/Dev/rust_prism/test_files/sphere.ply";

fn main() {
    let mut mesh = fileio::ply::load_mesh(MODEL).unwrap();
    mesh.create_embree_geometry();
    //let mesh_pos = Transf::new_translate(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
    //mesh.transform(mesh_pos);

    //let mesh_ref = scene::allocate_mesh(mesh);

    let mut scene = scene::Scene::new();
    let mesh_ref = scene.add_to_geom_pool(mesh); //mesh);
    scene.add_toplevel_geom(mesh_ref, 0);
    scene.build_scene();

    // let ray = pmath::ray::Ray::new(
    //     Vec3 {
    //         x: -3.0,
    //         y: 0.0,
    //         z: 0.0,
    //     },
    //     Vec3 {
    //         x: 1.0,
    //         y: 0.0,
    //         z: 0.0,
    //     },
    //     1.0,
    // );
    // println!("{:#?}", scene.intersect(ray));
    // return;

    let camera_pos = Transf::new_lookat(
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3 {
            x: -2.0,
            y: 0.0,
            z: 0.0,
        },
    );

    let bbox = pmath::bbox::BBox2::from_pnts(Vec2 { x: -1.0, y: -1.0 }, Vec2 { x: 1.0, y: 1.0 });
    let cam = PerspectiveCamera::new(camera_pos, 90.0, 0.0, 1.0, bbox, Vec2 { x: 400, y: 400 });

    let filter = filter::GaussianFilter::new(Vec2 { x: 1.0, y: 1.0 }, 0.5);
    let pixel_filter = filter::PixelFilter::new(&filter);
    let param = threading::RenderParam {
        num_pixel_samples: 5,
        num_threads: 64,
        sample_seed: 13,
        blue_noise_count: 3,
        res: Vec2 { x: 400, y: 400 },
    };
    let now = Instant::now();
    let film = threading::render::<NormalIntegrator, NormalIntegratorManager>(
        &cam,
        pixel_filter,
        &scene,
        param,
        true,
    )
    .unwrap();
    println!("Render time: {}", now.elapsed().as_nanos());

    let image_buffer = film.to_image_buffer(|color| film::ImagePixel {
        r: color.r,
        g: color.g,
        b: color.b,
    });

    film::png::write_png(&image_buffer, "normal2.png", film::png::BitDepth::EIGHT).unwrap();
}
