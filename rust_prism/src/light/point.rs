use crate::light::many_lights::LightCone;
use crate::light::{Light, LightType};
use crate::spectrum::Color;
use pmath::numbers::Float;
use pmath::vector::{Vec2, Vec3};

/// A point light source.
pub struct Point {
    intensity: Color,
}

impl Point {
    const LIGHT_TYPE: LightType = LightType::DELTA_POSITION;

    pub fn new(intensity: Color) -> Self {
        Point { intensity }
    }
}

impl Light for Point {
    fn sample(&self, surface_point: Vec3<f64>, _: f64, u: Vec2<f64>) -> (Color, Vec3<f64>, f64) {
        let dist_sqrt = (-surface_point).length2();
        (self.intensity.div_scale(dist_sqrt), Vec3::zero(), 1.)
    }

    fn pdf(&self, _: Vec3<f64>, _: Vec3<f64>) -> f64 {
        // It is practically impossible to get pick the correct direction in this case:
        0.
    }

    fn power(&self) -> Color {
        self.intensity.scale(f64::PI * 4.)
    }

    fn is_delta(&self) -> bool {
        Self::LIGHT_TYPE.contains(LightType::DELTA_POSITION)
            || Self::LIGHT_TYPE.contains(LightType::DELTA_DIRECTION)
    }

    fn get_cone(&self) -> LightCone {
        todo!();
    }
}
