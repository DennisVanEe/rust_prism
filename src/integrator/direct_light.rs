use super::{Integrator, SamplerParam};

use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::camera;

// Specifies how to sample lights (either sample all lights (splitting) or sample
// just one light):
pub enum LightTechnique {
    // Sample all of the lights in the scene
    ALL,
    // Only sample a single light
    ONE,
}

// A very basic integrator that only takes into account direct light and nothing else.
// Not really the most exciting thing in the world. Useful for quick testing of things
// though.
pub struct DirectLight<'a, S: Sampler, Camera: camera::Camera> {
    sampler: S,                      // The sampler used by the integrator
    camera: &'a Camera,              // The camera used for rendering
    light_technique: LightTechnique, // Specifies the type of light integration technique
    max_depth: u32,                  // Specifies the maximum depth of the integrator
}

impl<'a, S: Sampler, Camera: camera::Camera> DirectLight<'a, S, Camera> {
    pub fn new(
        ex_sampler_param: S::ParamType,   // Extra sampler parameters that might be needed
        def_sampler_param: SamplerParam,  // Default sampler parameters
        
        scene: &Scene, 
        camera: &'a Camera,
        light_technique: LightTechnique,
        max_depth: u32,
    ) -> Self {

        let scene_lights = scene.get_lights();

        // Specify the light sample counts:
        let arr_sizes_2d = if let LightTechnique::ALL = light_technique {
            let arr_sizes_2d = Vec::with_capacity(2 * scene_lights.len());
            for scene_light in scene_lights {
                let sample_count = S::round_count(scene_light.light.num_samples());
                arr_sizes_2d.push(sample_count);
                arr_sizes_2d.push(sample_count);
            }
            arr_sizes_2d
        } else {
            // Don't allocate any memory otherwise:
            Vec::new()
        };

        let sampler = S::new(
            ex_sampler_param,
            def_sampler_param.num_pixel_samples,
            def_sampler_param.num_dim,
            def_sampler_param.seed,
            &[],
            &arr_sizes_2d[..],
        );

        DirectLight {
            sampler,
            camera,
            light_technique,
            max_depth,
        }
    }
}

impl<S: Sampler> Integrator for DirectLight<S> {
    fn render(&mut self, scene: &Scene) -> Spectrum {
        todo!()
    }
}
