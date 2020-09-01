use crate::shading::lobe::{Lobe, LobeType};
use crate::spectrum::Color;
use pmath::numbers::Float;
use pmath::vector::Vec3;

//
// Lamabertian Reflection
//
// A very basic diffuse surface. It has the same brightness from any angle you look at it.

pub struct LambertianReflection {
    r_scale: Color,
}

impl LambertianReflection {
    const LOBE_TYPE: LobeType = LobeType::REFLECTION | LobeType::DIFFUSE;

    pub fn new(r_scale: Color) -> Self {
        LambertianReflection { r_scale }
    }
}

impl Lobe for LambertianReflection {
    // Checks if the types are contained
    fn contains_type(&self, lobe_type: LobeType) -> bool {
        Self::LOBE_TYPE.contains(lobe_type)
    }

    fn get_type(&self) -> LobeType {
        Self::LOBE_TYPE
    }

    fn eval(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> Color {
        self.r_scale.scale(f64::INV_PI)
    }

    // fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>]) -> RGBSpectrum {
    //     self.r_scale
    // }

    // fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>]) -> RGBSpectrum {
    //     self.r_scale
    // }
}

//
// Lamabertian Transmission
//

pub struct LambertianTransmission {
    t_scale: Color,
}

impl LambertianTransmission {
    const LOBE_TYPE: LobeType = LobeType::REFLECTION | LobeType::DIFFUSE;

    pub fn new(t_scale: Color) -> Self {
        LambertianTransmission { t_scale }
    }
}

impl Lobe for LambertianTransmission {
    fn contains_type(&self, lobe_type: LobeType) -> bool {
        Self::LOBE_TYPE.contains(lobe_type)
    }

    fn get_type(&self) -> LobeType {
        Self::LOBE_TYPE
    }

    fn eval(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> Color {
        self.t_scale.scale(f64::INV_PI)
    }

    // fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>]) -> RGBSpectrum {
    //     self.t_scale
    // }

    // fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>]) -> RGBSpectrum {
    //     self.t_scale
    // }
}
