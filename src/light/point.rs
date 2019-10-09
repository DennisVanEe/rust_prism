use crate::light::{Light, LightType};
use crate::math::numbers::Float;
use crate::math::vector::{Vec2, Vec3};
use crate::spectrum::RGBSpectrum;

pub struct Point {
    intensity: RGBSpectrum,
}

impl Point {
    const LIGHT_TYPE: LightType = LightType::DELTA_POSITION;

    pub fn new(intensity: RGBSpectrum) -> Self {
        Point { intensity }
    }
}

impl Light for Point {
    fn sample(
        &self,
        surface_point: Vec3<f64>,
        _: f64,
        u: Vec2<f64>,
    ) -> (f64, RGBSpectrum, Vec3<f64>) {
        let distSqrt = (-surface_point).length2();
        (1., self.intensity.div_scale(distSqrt), Vec3::zero())
    }

    fn power(&self) -> RGBSpectrum {
        self.intensity.scale(f64::PI * 4.)
    }
}
