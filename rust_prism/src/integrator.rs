use crate::film::Pixel;
use crate::math::ray::{PrimaryRay, Ray, RayDiff};
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;

/// Basic test integrator.
pub fn integrate(
    prim_ray: PrimaryRay<f64>,
    scene: &Scene,
    _: &mut Sampler,
    _: u32,
    pixel: &mut Pixel,
) {
    //println!("{:#?}", prim_ray.ray);
    if let None = scene.intersect(prim_ray.ray) {
        pixel.add_sample(Spectrum::black());
    } else {
        pixel.add_sample(Spectrum::white());
    }
}

// pub fn integrate(
//     prim_ray: PrimaryRay<f64>,
//     scene: &Scene,
//     sampler: &mut Sampler,
//     max_depth: u32,
//     pixel: &mut Pixel,
// ) {
//     let mut spectrum = Spectrum::black();
//     let mut specular_bounce = false;
//     let mut ray = prim_ray.ray;

//     for bounce in 0..max_depth {
//         // Intersect the scene and see what we hit.
//         let interaction = match scene.intersect(ray) {
//             Some(int) => int,
//             _ => break,
//         };

//         // Need to perform this check in case the ray hits an emissive object.
//         // That is, account for light if it's a primary ray or the last operation
//         // was a specular bounce.
//         if bounce == 0 || specular_bounce {}
//     }

//     todo!();
// }
