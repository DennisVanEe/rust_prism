// Clean this stuff up in the future...
// This is here just for now.

#![allow(dead_code)]

mod camera;
mod embree;
mod film;
mod filter;
mod integrator;
mod loading;
mod math;
mod mesh;
mod sampler;
mod scene;
mod spectrum;
mod threading;
mod transform;

use camera::perspective::PerspectiveCamera;
use math::vector::{Vec2, Vec3};
use transform::Transf;

use std::time::Instant;

const MODEL: &'static str = "/home/dennis/Dev/rust_prism/test_files/sphere.ply";

fn main() {
    embree::DEVICE.create_device("");
    let mut mesh = loading::ply::load_mesh(MODEL).unwrap();
    mesh.create_embree_geometry();
    //let mesh_pos = Transf::new_translate(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
    //mesh.transform(mesh_pos);

    let mesh_ref = scene::allocate_mesh(mesh);

    let mut scene = scene::Scene::new();
    scene.add_toplevel_mesh(mesh_ref, 0);
    scene.build_scene();

    // let ray = math::ray::Ray::new(Vec3 {x: -3.0, y: 0.0, z: 0.0 }, Vec3 { x: 1.0, y: 0.0, z: 0.0 }, 1.0);
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

    let bbox = math::bbox::BBox2::from_pnts(Vec2 { x: -1.0, y: -1.0 }, Vec2 { x: 1.0, y: 1.0 });
    let cam = PerspectiveCamera::new(
        camera_pos,
        90.0,
        0.0,
        1.0,
        0.0,
        1.0,
        bbox,
        Vec2 { x: 400, y: 400 },
    );

    let filter = filter::GaussianFilter::new(Vec2 { x: 1.0, y: 1.0 }, 0.5);
    let pixel_filter = filter::PixelFilter::new(&filter);
    let param = threading::RenderParam {
        max_depth: 1,
        num_pixel_samples: 10,
        num_threads: 1,
        res: Vec2 { x: 400, y: 400 },
    };
    let now = Instant::now();
    let film = threading::render(&cam, pixel_filter, &scene, param);
    println!("Render time: {}", now.elapsed().as_nanos());

    let image_buffer = film.to_image_buffer(|color| film::ImagePixel {
        r: color.r,
        g: color.g,
        b: color.b,
    });

    film::png::write_png(&image_buffer, "test.png", film::png::BitDepth::EIGHT).unwrap();
}
