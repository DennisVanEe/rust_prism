use crate::shading::lobe::lambertian::LambertianReflection;
use crate::shading::lobe::oren_nayar::OrenNayar;
use crate::shading::lobe::Lobe;
use crate::shading::material::{Bsdf, Material, MaterialConstructor, MaterialPool};
use crate::spectrum::RGBSpectrum;

// This uses a the same value for these properties across the entire
// surface of the model. If you want this to be parametarized, see
// MatteTexture.
pub struct Matte {
    color: RGBSpectrum,
    sigma: f64,
}

impl Matte {
    pub fn new(color: RGBSpectrum, sigma: f64) -> Self {
        Matte { color, sigma }
    }
}

impl MaterialConstructor for Matte {
    fn new_material<'a>(&self, pool: &mut MaterialPool<'a>) {
        let bsdf = if self.color.is_black() {
            Bsdf::new(1., &[])
        } else {
            // If sigma is 0, then we can use the cheaper lambertian material:
            let lobes: [&'a dyn Lobe; 1] = if self.sigma == 0. {
                // Allocate a lambertian lobe:
                [pool.add_lobe(LambertianReflection::new(self.color))]
            } else {
                // Allocate a fancy OrenNayar lobe:
                [pool.add_lobe(OrenNayar::new(self.color, self.sigma))]
            };
            Bsdf::new(1., &lobes)
        };
        pool.add_material(Material::new(bsdf));
    }
}

// TODO: Once I add texture support to the renderer, then a matte material
// can gather it's information from a texture.
pub struct MatteTexture {}
