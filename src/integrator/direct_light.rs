use super::{Integrator, RenderParam, IntegratorParam};

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
    sampler: S,                     
    camera: &'a Camera,      
    light_technique: LightTechnique,
    max_depth: u32,
}

pub struct DirectLightParam {
    pub light_technique: LightTechnique,
    
}

impl<'a, S: Sampler, Camera: camera::Camera> DirectLight<'a, S, Camera> {
    type Param = DirectLightParam;

    pub fn new(param: DirectLightParam, int_param: IntegratorParam) -> Self {
        let light_technique = param.light_technique;

        // Go through and prepare all of the samples for all of the lights in the scene:

        let scene_lights = int_param.scene.get_lights();

        let arr_sizes_2d = if let LightTechnique::ALL = param.light_technique {
            let arr_sizes_2d = Vec::with_capacity(2 * scene_lights.len());
            for scene_light in scene_lights {
                let sample_count = S::round_count(scene_light.light.num_samples());
                arr_sizes_2d.push(sample_count);
                arr_sizes_2d.push(sample_count);
            }
            arr_sizes_2d
        } else {
            Vec::new()
        };

        let sampler = S::new(
            int_param.sampler_param,
            int_param.num_pixel_samples,
            int_param.num_dim,
            &[],
            &arr_sizes_2d[..]
        );

        DirectLight {
            sampler,
            camera: int_param.camera,
            light_technique: param.light_technique,
            max_depth: int_param.max_depth,
        }
    }
}

impl<'a, S: Sampler> Integrator for DirectLight<'a, S> {
    fn render(&mut self, param: RenderParam) {
        
    }
}
