use super::{Integrator, IntegratorParam, RenderParam};

use crate::camera::Camera;
use crate::sampler::Sampler;
use crate::scene::Scene;

// Specifies how to sample lights (either sample all lights (splitting) or sample
// just one light):
#[derive(Clone, Copy)]
pub enum LightTechnique {
    // Sample all of the lights in the scene
    ALL,
    // Only sample a single light
    ONE,
}

// Extra parameters that the DirectLight integrator may need:
pub struct DirectLightParam {
    pub light_technique: LightTechnique,
}

// A very basic integrator that only takes into account direct light and nothing else.
// Not really the most exciting thing in the world. Useful for quick testing of things
// though.
#[derive(Clone)]
pub struct DirectLight<'a, S: Sampler, C: Camera> {
    camera: &'a C,
    light_technique: LightTechnique,
    max_depth: u32,
}

impl<'a, S: Sampler, C: Camera> DirectLight<'a, S, C> {
    type Param = DirectLightParam;

    pub fn new(param: DirectLightParam, int_param: IntegratorParam) -> Self {
        
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

        int_param.sampler.prepare_arrays(&[], &arr_sizes_2d[..]);

        DirectLight {
            camera: int_param.camera,
            light_technique: param.light_technique,
            max_depth: int_param.max_depth,
        }
    }
}

impl<'a, S: Sampler> Integrator for DirectLight<'a, S> {
    fn render(&mut self, param: RenderParam) {}
}
