use crate::light::light_picker::{LightPicker, LightPickerManager};
use crate::sampler::Sampler;
use crate::scene::Scene;
use pmath::vector::Vec3;

pub struct UniformAllManager {}

impl LightPickerManager<UniformAll> for UniformAllManager {
    fn spawn_lightpicker(&self, _thread_id: u32) -> UniformAll {
        UniformAll::new()
    }
}

pub struct UniformAll {
    max_num_lights: usize,
    curr_light: usize,
}

/// This one just uniformly samples all available lights in a scene.
impl UniformAll {
    pub fn new() -> Self {
        UniformAll {
            max_num_lights: 0,
            curr_light: 0,
        }
    }
}

impl LightPicker for UniformAll {
    fn set_scene_lights(&mut self, num_lights: usize, _scene: &Scene) {
        self.max_num_lights = num_lights;
    }

    fn pick_lights<'a>(
        &mut self,
        _shading_point: Vec3<f64>,
        _normal: Vec3<f64>,
        _sampler: &mut Sampler,
        _scene: &Scene,
    ) {
        // Fairly straight forward as it just goes through all of the lights uniformly
        self.curr_light = 0;
    }

    fn get_next_light(&mut self) -> Option<(usize, f64)> {
        if self.curr_light >= self.max_num_lights {
            return None;
        }
        let result = Some((self.curr_light, 1.0));
        self.curr_light += 1;
        result
    }
}
