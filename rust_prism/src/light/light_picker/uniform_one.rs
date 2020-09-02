use crate::light::light_picker::LightPicker;
use crate::sampler::Sampler;
use crate::scene::Scene;
use pmath::vector::Vec3;

pub struct UniformOne {
    max_num_lights: u32,
}

impl UniformOne {
    pub fn new() -> Self {
        UniformOne { max_num_lights: 0 }
    }
}

impl LightPicker<UniformOneIter> for UniformOne {
    fn set_scene_lights(&mut self, num_lights: u32, _scene: &Scene) {
        self.max_num_lights = num_lights;
    }

    fn pick_lights<'a>(
        &mut self,
        _shading_point: Vec3<f64>,
        _normal: Vec3<f64>,
        sampler: &mut Sampler,
        _scene: &Scene,
    ) -> UniformOneIter {
        let u = sampler.sample().x;
        let picked_light =
            Some(((u * (self.max_num_lights as f64)) as usize).min(self.max_num_lights - 1));
        UniformOneIter {
            picked_light,
            max_num_lights: self.max_num_lights as f64,
        }
    }
}

pub struct UniformOneIter {
    picked_light: Option<u32>,
    max_num_lights: f64,
}

impl Iterator for UniformOneIter {
    type Item = (u32, f64);

    fn next(&mut self) -> Option<(u32, f64)> {
        match self.picked_light {
            Some(light) => {
                let result = (light, self.max_num_lights);
                self.picked_light = None;
                result
            }
            None => None,
        }
    }
}
