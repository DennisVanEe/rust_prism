pub mod direct_light;

use crate::geometry::GeomInteraction;
use crate::light::Light;
use crate::math::ray::Ray;
use crate::math::vector::Vec2;
use crate::math::numbers::Float;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneLight, SceneObjectType};
use crate::shading::lobe::LobeType;
use crate::shading::material::Bsdf;
use crate::spectrum::Spectrum;
use crate::film::{Film, TileIndex};

use std::f64;

// These are parameters provided to every sampler
// that are exposed to the public (that is, a user defined
// setting):
pub struct SamplerParam {
    num_pixel_samples: usize,
    num_dim: usize,
    seed: u64,
}

// TODO: Think about how to make an interface for the film and aov buffer system so that
// it can interface with a generic integrator in the best way possible (and so that the
// integrator doesn't repeat too much doe).

// Each thread gets its own integrator and tile. So, during rendering,
// a thread would fill up a tile with values and, when it's done, it'll move
// onto the next tile. It'll keep doing this until all tiles have been rendered
// NOTE: adaptive sampling is something I will add later. It's on my todo list!
// NOTE: If a custom integrator needs any extra stuff, just add it to the constructor
pub trait Integrator<'a> {
    // Certain integrators may require certain aov buffers to exist regardless of user
    // demand (like a variance buffer). All required buffers will be created here with
    // the given tile resolution. These will override user film buffer settings:
    fn req_film(&self, tile_res: Vec2<usize>) -> Film {
        // Default to not requiring anything:
        Film::new()
    }

    // Given an immutable film, check what aov buffers are available, and get scene information
    // as well:
    fn preprocess(&mut self, film: &'a Film, scene: &Scene);

    // This function goes ahead and renders a single tile with the given tile.
    // Once it's done it must return the TileIndex it used as some TileSchedulers
    // find this information important:
    fn render(&mut self, index: TileIndex, scene: &Scene) -> TileIndex;
}

// Calculates the balance heurisitc for the first distribution provided out of the two:
fn balance_heuristic(num_samples: usize, pdf: f64, num_samples_o: usize, pdf_o: f64) -> f64 {
    let num_samples = num_samples as f64;
    let num_samples_o = num_samples_o as f64;
    (num_samples * pdf) / (num_samples * pdf + num_samples_o + pdf_o)
}

// Calculates the power heurisitc for the first distribution provided out of the two.
// Here, the power (beta) is 2:
fn power2_heuristic(num_samples: usize, pdf: f64, num_samples_o: usize, pdf_o: f64) -> f64 {
    let num_samples = num_samples as f64;
    let num_samples_o = num_samples_o as f64;
    let s = num_samples * pdf;
    let s_o = num_samples_o * pdf_o;
    (s * s) / (s * s + s_o * s_o)
}

// Essentially we are performing MIS on both the BSDF and light distribution with the given bsdf and light:
fn estimate_direct(
    int: GeomInteraction,      // Specifies the interaction at the point where we intersected the object (scene space).
    bsdf: &Bsdf,               // The bsdf (material) at the point that we care about.
    curr_time: f64,            // The time when we are performing this test (used for things like shadows).
    scene: &Scene,             // The scene where all of this is taking place.
    light_sample: Vec2<f64>,   // Sample used to sample the light (if area light).
    bsdf_sample: Vec2<f64>,    // Sample used to sample the bsdf.
    scene_light: &SceneLight,  // The light we are sampling from.
) -> Spectrum {

    let (light_result, light_pos, light_pdf) = scene_light.sample(int.p, curr_time, light_sample);
    // wi points away from the surface and is normalized:
    let wi = (light_pos - int.p).normalize();

    // Now we check whether or not it's occluded:
    if scene.intersect_test(
        Ray {
            org: int.p + wi.scale(f64::SELF_INT_COMP),
            dir: wi,
        },
        f64::INFINITY,
        curr_time,
    ) {
        return Spectrum::black();
    }

    // Calculate how much the light contributes in this case. Sampling it only once.
    let light_contrib = if light_pdf > 0. && !light_result.is_black() {
        // TODO: figure out what lobe flags we should use here:
        // Evaulate the bsdf at the surface:
        let bsdf_result = bsdf
            .eval(int.wo, wi, LobeType::ALL)
            .scale(wi.dot(int.shading_n).abs());
        let bsdf_pdf = bsdf.pdf(int.wo, wi, LobeType::ALL);

        // Check if the light is a "delta light". This is a special case that
        // always returns 1 for the pdf. If that is the case, we don't have to
        // worry about specifying a certain weight:
        if scene_light.light.is_delta() {
            // Normal monte carlo estimator (weight = 1):
            bsdf_result * light_result.div_scale(light_pdf)
        } else {
            // MIS monte carlo estimator:
            let mis_w = power2_heuristic(1, light_pdf, 1, bsdf_pdf);
            (bsdf_result * light_result).scale(mis_w / light_pdf)
        }
    } else {
        Spectrum::black()
    };

    // Now we see how much the bsdf contributes:
    let bsdf_contrib = if !scene_light.light.is_delta() {
        // Sample the bsdf (TODO: figure out the LobeType flag).
        let (bsdf_result, bsdf_wi, bsdf_pdf, lobe_type) =
            bsdf.sample(int.wo, bsdf_sample, LobeType::ALL);

        if !bsdf_result.is_black() && bsdf_pdf > 0. {
            let bsdf_result = bsdf_result.scale(bsdf_wi.dot(int.shading_n).abs());
            // Only bother sampling the light if the lobe isn't specular. If it is,
            // it's unlikely we will hit it, so the weight can be set to 1:
            let mis_w = if !lobe_type.contains(LobeType::SPECULAR) {
                let light_pdf = scene_light.light.pdf(int.p, bsdf_wi);
                // If light_pdf is 0, then there is no need to do any further calculations.
                // Otherwise, we can go ahead and calculate the power2 heuristic:
                if light_pdf == 0. {
                    return light_contrib;
                }
                power2_heuristic(1, bsdf_pdf, 1, light_pdf)
            } else {
                1.
            };

            // Now we need to check whether or not hiting this specific light contributes anything:
            let ray = Ray {
                org: int.p + bsdf_wi.scale(f64::SELF_INT_COMP),
                dir: bsdf_wi,
            };
            let scene_int = scene.intersect(ray, f64::INFINITY, curr_time);
            let light_result = if let Some(i) = scene_int {
                // Check if the thing we hit is a light:
                match i.obj_type {
                    SceneObjectType::Light(l) if mem::is_ptr_same(l, scene_light.light) => i.light_radiance(-wi),
                    _ => Spectrum::black()
                }
            } else {
                // TODO: add support for area lights here (that is, if the "light" argument
                // is an area light, we need to make sure that it contributes regardless):
                Spectrum::black()
            };

            (light_result * bsdf_result).scale(mis_w / bsdf_pdf)

        // Contribute nothing if the bsdf result is zero, so we can just return
        // from the given function:
        } else {
            return light_contrib;
        }
    } else {
        // If it's a delta distribution, then don't bother contributing
        // anything from the bsdf as there is no way we'll hit it:
       return light_contrib;
    };

    light_contrib + bsdf_contrib
}

// Some important functions that may be useful for all integrators:

// This is an integrator that uniformly samples all lights in a scene:
fn uniform_sample_all_lights<S: Sampler>(
    int: GeomInteraction,        // Point from which we are sampling
    bsdf: &Bsdf,                 // The Bsdf at the point from which we are sampling
    curr_time: f64,              // The current time used for moving objects and whatnot
    scene: &Scene,               // The scene where the intersection is occuring
    scene_lights: &[SceneLight], // The lights we are sampling
    sampler: &mut S,             // The Sampler we are using to sample values
) -> Spectrum {
    // Loop over all the lights in the scene here:
    scene_lights.iter().fold(Spectrum::black(), |total, curr_light| {
        // Don't worry about scattering media for now:
        let light_samples = sampler.get_2d_array();
        let bsdf_samples = sampler.get_2d_array();

        // Check if the sampler has any samples left:
        if light_samples.is_empty() || bsdf_samples.is_empty() {
            let light_sample = sampler.get_2d();
            let bsdf_sample = sampler.get_2d();
            total + estimate_direct(int, bsdf, curr_time, scene, light_sample, bsdf_sample, curr_light)
        // If it does, then go through and sample each of them:
        } else {
            let sum_samples = light_samples
                .iter()
                .zip(bsdf_samples)
                .fold(Spectrum::black(), |total, (&light_sample, &bsdf_sample)| {
                    total + estimate_direct(int, bsdf, curr_time, scene, light_sample, bsdf_sample, curr_light)
                });
            // The length of both light_samples and bsdf_samples should be the same:
            total + (sum_samples.div_scale(light_samples.len() as f64))
        }
    })
}

fn uniform_sample_one_light<S: Sampler>(
    int: GeomInteraction,        // Point from which we are sampling
    bsdf: &Bsdf,                 // The Bsdf at the point from which we are sampling
    curr_time: f64,              // The current time used for moving objects and whatnot
    scene: &Scene,               // The scene that we are working on
    scene_lights: &[SceneLight], // All of the lights in the scene
    sampler: &mut S,             // The Sampler we are using to sample values
) -> Spectrum {

    // Check if we have any lights in the scene at all:
    if scene_lights.is_empty() {
        return Spectrum::black();
    }

    let num_lights = scene_lights.len() as f64;

    // Randomly pick a light:
    let light = {
        let sample = sampler.get_1d();
        let light_index = (scene_lights.len() - 1).min((num_lights * sample) as usize);
        &scene_lights[light_index]
    };

    let light_sample = sampler.get_2d();
    let bsdf_sample = sampler.get_2d();
    estimate_direct(int, bsdf, curr_time, scene, light_sample, bsdf_sample, light).scale(num_lights)
}
