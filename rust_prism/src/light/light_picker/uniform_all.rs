use crate::light::light_picker::LightPicker;
use crate::sampler::Sampler;
use crate::scene::Scene;
use pmath::vector::Vec3;

pub struct UniformAll {
    max_num_lights: u32,
}

/// This one just uniformly samples all available lights in a scene.
impl UniformAll {
    pub fn new() -> Self {
        UniformAll { max_num_lights: 0 }
    }
}

impl LightPicker<UniformAllIter> for UniformAll {
    fn set_scene_lights(&mut self, num_lights: usize, _scene: &Scene) {
        self.max_num_lights = num_lights;
    }

    fn pick_lights<'a>(
        &mut self,
        _shading_point: Vec3<f64>,
        _normal: Vec3<f64>,
        _sampler: &mut Sampler,
        _scene: &Scene,
    ) -> UniformAllIter {
        // Fairly straight forward as it just goes through all of the lights uniformly
        UniformAllIter {
            max_num_lights: self.max_num_lights,
            curr_light_num: 0,
        }
    }
}

pub struct UniformAllIter {
    max_num_lights: u32,
    curr_light_num: u32,
}

impl Iterator for UniformAllIter {
    type Item = (u32, f64);

    fn next(&mut self) -> Option<(u32, f64)> {
        if self.curr_light_num < self.max_num_lights {
            self.curr_light_num += 1;
            Some((self.curr_light_num - 1, 1.0))
        } else {
            None
        }
    }
}
