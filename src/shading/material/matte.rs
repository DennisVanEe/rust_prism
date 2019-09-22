use crate::shading::material::{Bsdf, Material};
use crate::geometry::Interaction;
use crate::memory::allocators::DynStackAlloc;
use crate::shading::lobe::Lobe;
use crate::shading::lobe::oren_nayar::OrenNayar;
use crate::shading::lobe::lambertian::LambertianReflection;
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
        Matte {
            color,
            sigma,
        }
    }
}

impl Material for Matte {
    fn compute_bsdf<'a>(
        &self,
        interaction: Interaction,
        memory: &'a DynStackAlloc,
        use_multiple_lobes: bool,
    ) -> Bsdf<'a> {
        // Check if the color is black or not. If it is black,
        // then it has no lobes:
        if self.color.is_black() {
            Bsdf::new_empty(interaction, 1.)
        } else {
            // If sigma is 0, then we can use the cheaper lambertian material:
            let lobes = if self.sigma == 0. {
                // Allocate a lambertian lobe:
                [memory.push(LambertianReflection::new(self.color)) as &'a dyn Lobe]
            } else {
                // Allocate a fancy OrenNayar lobe:
                [memory.push(OrenNayar::new(self.color, self.sigma)) as &'a dyn Lobe]
            };
            Bsdf::new(interaction, 1., &lobes)
        }
    }
}

// TODO: Once I add texture support to the renderer, then a matte material
// can gather it's information from a texture.
pub struct MatteTexture {}
