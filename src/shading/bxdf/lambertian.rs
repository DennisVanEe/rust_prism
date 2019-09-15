use crate::math::numbers::Float;
use crate::math::vector::{Vec2, Vec3};
use crate::shading::bxdf::{BxDF, BxDFType};
use crate::spectrum::RGBSpectrum;

//
// Lamabertian Reflection
//
// A very basic diffuse surface. It has the same brightness from any angle you look at it.

pub struct LambertianReflection {
    r_scale: RGBSpectrum,
}

impl LambertianReflection {
    const BXDF_TYPE: BxDFType = BxDFType::REFLECTION | BxDFType::DIFFUSE;

    pub fn new(r_scale: RGBSpectrum) -> Self {
        LambertianReflection { r_scale }
    }
}

impl BxDF for LambertianReflection {
    fn has_flags(&self, fl: BxDFType) -> bool {
        Self::BXDF_TYPE.contains(fl)
    }

    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
        self.r_scale.scale(f64::INV_PI)
    }

    fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>]) -> RGBSpectrum {
        self.r_scale
    }

    fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>]) -> RGBSpectrum {
        self.r_scale
    }
}

//
// Lamabertian Transmission
//

pub struct LambertianTransmission {
    t_scale: RGBSpectrum,
}

impl LambertianTransmission {
    const BXDF_TYPE: BxDFType = BxDFType::TRANSMISSION | BxDFType::DIFFUSE;

    pub fn new(t_scale: RGBSpectrum) -> Self {
        LambertianTransmission { t_scale }
    }
}

impl BxDF for LambertianTransmission {
    fn has_flags(&self, fl: BxDFType) -> bool {
        Self::BXDF_TYPE.contains(fl)
    }

    fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>) -> RGBSpectrum {
        self.t_scale.scale(f64::INV_PI)
    }

    fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>]) -> RGBSpectrum {
        self.t_scale
    }

    fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>]) -> RGBSpectrum {
        self.t_scale
    }
}
