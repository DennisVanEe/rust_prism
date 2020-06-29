// Need to add Microfacet support for this to work properly. So yeah

// use crate::geometry::Interaction;
// use crate::memory::allocators::DynStackAlloc;
// use crate::shading::lobe::lambertian::LambertianReflection;
// use crate::shading::lobe::Lobe;
// use crate::shading::material::{Bsdf, Material};
// use crate::spectrum::RGBSpectrum;

// pub struct Plastic {
//     color_diffuse: RGBSpectrum,
//     color_reflect: RGBSpectrum,
//     roughness: f64,
// }

// impl Plastic {
//     pub fn new(color_diffuse: RGBSpectrum, color_reflect: RGBSpectrum, roughness: f64) -> Self {
//         Plastic {
//             color_diffuse,
//             color_reflect,
//             roughness,
//         }
//     }
// }

// impl Material for Plastic {
//     // I'll worry about generating bssrdfs later:
//     fn compute_bsdf<'a>(
//         &self,
//         interaction: Interaction,
//         memory: &'a DynStackAlloc,
//         use_multiple_lobes: bool,
//     ) -> Bsdf<'a> {
//         let mut bsdf = Bsdf::new_empty(interaction, 1.);
//         // First we need to check how many lobes we should allocate:
//         if self.color_diffuse != 0. {
//             let lobe = memory.push(LambertianReflection::new(self.color_diffuse)) as &'a dyn Lobe;
//             bsdf.add_lobe(lobe);
//         }

//         if self.color_reflect != 0. {
//             // First, we need some sort of fresnel dielectric:
//             let fresnel =  Dielectric::new(1., 1.5);

//         }
//     }
// }
