use crate::light::light_picker::LightPicker;
use crate::sampler::Sampler;
use crate::scene::Scene;
use pmath::vector::Vec3;

pub struct UniformOne {
    max_num_lights: usize,
    picked_light: Option<usize>,
}

impl UniformOne {
    pub fn new() -> Self {
        UniformOne {
            max_num_lights: 0,
            picked_light: None,
        }
    }
}

impl LightPicker for UniformOne {
    fn set_scene_lights(&mut self, num_lights: usize, _scene: &Scene) {
        self.max_num_lights = num_lights;
    }

    fn pick_lights<'a>(
        &mut self,
        _shading_point: Vec3<f64>,
        _normal: Vec3<f64>,
        sampler: &mut Sampler,
        _scene: &Scene,
    ) {
        let u = sampler.sample().x;
        self.picked_light =
            Some(((u * (self.max_num_lights as f64)) as usize).min(self.max_num_lights - 1));
    }

    fn get_next_light(&mut self) -> Option<(usize, f64)> {
        match self.picked_light {
            Some(light) => {
                self.picked_light = None;
                Some((light, self.max_num_lights as f64))
            }
            None => None,
        }
    }
}
