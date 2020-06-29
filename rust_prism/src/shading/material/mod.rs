pub mod matte;
pub mod plastic;

use crate::mesh::Interaction;
use crate::math::vector::{Vec2, Vec3};
use crate::shading::lobe::{Lobe, LobeType};
use crate::spectrum::Spectrum;
use arrayvec::ArrayVec;
use bumpalo::Bump;

// Manages all of the materials in a scene. Used to index materials given a
// material id:

// Ensures that the lobes have static lifetime
static LOBE_POOL: Bump = Bump::new();

pub struct MaterialPool<'a> {
    materials: Vec<Material<'a>>, // Stores all of the materials
}

impl<'a> MaterialPool<'a> {
    pub fn new() -> Self {
        MaterialPool {
            materials: Vec::new(),
        }
    }

    // Any lobes that may be required for the varying bsdfs in the design
    // can add there stuff here:
    pub fn add_lobe<T: Lobe + 'a>(&self, lobe: T) -> &'a dyn Lobe {
        LOBE_POOL.alloc_with(|| lobe) as &'a dyn Lobe
    }

    pub fn add_material(&mut self, material: Material<'a>) -> u32 {
        let material_id = self.materials.len() as u32;
        self.materials.push(material);
        material_id
    }

    // Need to ensure that the material_id is correct and whatnot
    pub unsafe fn get_material(&self, material_id: u32) -> &'a Material {
        self.materials.get_unchecked(material_id as usize)
    }
}

// Just a bsdf and bssrdf
pub struct Material<'a> {
    bsdf: Bsdf<'a>,
    // TODO: add support for bssrdf
}

impl<'a> Material<'a> {
    pub fn new(bsdf: Bsdf<'a>) -> Self {
        Material {
            bsdf,
        }
    }

    pub fn get_bsdf(&self) -> &'a Bsdf {
        &self.bsdf
    }
}

// Used to construct a material and add it to the MaterialPool:
pub trait MaterialConstructor {
    fn new_material(&self, pool: &mut MaterialPool);
}

// Used to convert to and from shading coordinate space:
#[derive(Clone, Copy)]
pub struct ShadingCoord {
    geometry_n: Vec3<f64>,
    n: Vec3<f64>,
    s: Vec3<f64>,
    t: Vec3<f64>,
}

impl ShadingCoord {
    pub fn new(interaction: Interaction) -> Self {
        let s = interaction.dpdu.normalize();
        ShadingCoord {
            geometry_n: interaction.n,
            n: interaction.shading_n,
            s,
            t: interaction.shading_n.cross(s),
        }
    }

    pub fn world_to_shading(self, v: Vec3<f64>) -> Vec3<f64> {
        Vec3 {
            x: v.dot(self.s),
            y: v.dot(self.t),
            z: v.dot(self.n),
        }
    }

    pub fn shading_to_world(self, v: Vec3<f64>) -> Vec3<f64> {
        Vec3 {
            x: (self.s.x * v.x) + (self.t.x * v.y) + (self.n.x * v.z),
            y: (self.s.y * v.x) + (self.t.y * v.y) + (self.n.y * v.z),
            z: (self.s.z * v.x) + (self.t.z * v.y) + (self.n.z * v.z),
        }
    }

    // wo and wi are in SHADING SPACE. Used to detect if the incoming direction
    // (wi) is coming from behind. This would mean the light shouldn't be incorporated
    pub fn is_reflect(self, wo: Vec3<f64>, wi: Vec3<f64>) -> bool {
        wo.dot(self.geometry_n) * wi.dot(self.geometry_n) > 0.0
    }
}

// Public so that anyone making materials has this information.
// This can be pretty important:
pub const MAX_NUM_LOBES: usize = 8;

#[derive(Clone)]
pub struct Bsdf<'a> {
    lobes: ArrayVec<[&'a dyn Lobe; MAX_NUM_LOBES]>,
    eta: f64,
}

impl<'a> Bsdf<'a> {
    pub fn new(eta: f64, in_lobes: &[&'a dyn Lobe]) -> Self {
        // Check if we can fit that many lobes:
        debug_assert!(in_lobes.len() <= MAX_NUM_LOBES);
        // ArrayVec has no clone_from_slice function, so we do it this way:
        let mut lobes = ArrayVec::<[_; MAX_NUM_LOBES]>::new();
        for &lobe in in_lobes {
            lobes.push(lobe);
        }

        Bsdf {
            lobes,
            eta,
        }
    }

    // Returns the number of lobes that have the specified lobe type:
    pub fn num_contains_type(&self, lobe_type: LobeType) -> usize {
        self.lobes.iter().fold(0, |count, &lobe| {
            if lobe.contains_type(lobe_type) {
                count + 1
            } else {
                count
            }
        })
    }

    // Both wo and wi here are in SHADING SPACE
    pub fn eval(&self, wo: Vec3<f64>, wi: Vec3<f64>, fl: LobeType, is_reflect: bool) -> Spectrum {
        self.lobes.iter().fold(Spectrum::black(), |f, lobe| {
            let matches = lobe.contains_type(fl);
            // Checks that, if it's reflected then we have a reflection lobe and if it's
            // not reflected we have a transmission lobe.
            let valid_direction = (is_reflect && lobe.contains_type(LobeType::REFLECTION))
            || (!is_reflect && lobe.contains_type(LobeType::TRANSMISSION));

            if matches && valid_direction {
                f + lobe.eval(wo, wi)
            } else {
                f // otherwise we do nothing
            }
        })
    }

    // Both wo and wi here are in SHADING SPACE
    pub fn pdf(&self, wo: Vec3<f64>, wi: Vec3<f64>, fl: LobeType) -> f64 {
        // We are essentially averaging the pdfs that match the flags:
        let (pdf, num_has_type) =
            self.lobes
                .iter()
                .fold((0.0, 0), |(pdf_sum, count), lobe| {
                    // Don't double count the lobe we sampled:
                    if lobe.contains_type(fl) {
                        (pdf_sum + lobe.pdf(wo, wi), count + 1)
                    } else {
                        (pdf_sum, count)
                    }
                });
        pdf / (num_has_type as f64)
    }

    // Returns, in the following order:
    // Resulting throughput, wi (world space), pdf, lobe type of lobe samples (as option, in case there is no lobe sampled):
    pub fn sample(
        &self,
        world_wo: Vec3<f64>,
        u: Vec2<f64>,
        lobe_type: LobeType,
    ) -> (Spectrum, Vec3<f64>, f64, LobeType) {
        let num_has_type = self.num_contains_type(lobe_type);
        if num_has_type == 0 {
            return (Spectrum::black(), Vec3::zero(), 0., LobeType::NONE);
        }
        // TODO: pick a wiser selection algorithm for lobes that are much more
        // likely to be called instead of using just a uniform approach:
        // Uniformly select a lobe:
        // We have the min in case u.x * num_has_type >= 1
        let selected_lobe_index =
            ((u.x * (num_has_type as f64)).floor() as usize).min(num_has_type - 1);
        // Now we loop over our lobes and pick the first one that is selected_lobe'th place:
        let mut curr_count = 0;
        let &selected_lobe = self
            .lobes
            .iter()
            .find(|lobe| {
                if lobe.contains_type(lobe_type) {
                    curr_count += 1;
                    if curr_count == selected_lobe_index {
                        return true;
                    }
                }

                false
            })
            .unwrap();

        // We still want to use u, so we have to remap it so that u can still
        // be between 0 and 1.
        let u = Vec2 {
            x: u.x * (num_has_type - selected_lobe_index) as f64,
            y: u.y,
        };

        // Sample the selected lobe for the wi value:
        let wo = self.world_to_shading(world_wo);
        let sampled_lobe_type = selected_lobe.get_type();
        let (throughput, wi, pdf) = selected_lobe.sample(wo, u);
        let world_wi = self.shading_to_world(wi);

        // Calculate the new pdf value if it isn't specular and there are multiple lobes.
        // For now it's merely the average of all of the pdfs of each of the lobes.
        // (Not specular because the pdf = 1 as it's a dirac delta function):
        // TODO: when changing the above for efficiency, make sure to modify this pdf value as well!
        let pdf = if !sampled_lobe_type.contains(LobeType::SPECULAR) && num_has_type > 1 {
            self.lobes.iter().fold(pdf, |pdf_sum, &lobe| {
                // Don't double count the lobe we sampled:
                if (lobe as *const dyn Lobe != selected_lobe as *const dyn Lobe)
                    && lobe.matches_type(lobe_type)
                {
                    pdf_sum + lobe.pdf(wo, wi)
                } else {
                    pdf_sum
                }
            }) / num_has_type as f64
        } else {
            pdf
        };

        // Now we calculate the throughput by summing the contributions from each of the lobes.
        // It's more efficient to do it this way than constantly calling the Bsdf's eval function:
        let throughput = if !sampled_lobe_type.contains(LobeType::SPECULAR) && num_has_type > 1 {
            // Check if they are on the same side relative to the normal (reflected):
            let is_reflect = world_wi.dot(self.geometry_n) * world_wo.dot(self.geometry_n) > 0.;
            self.lobes.iter().fold(throughput, |result, &lobe| {
                // Don't want to compute eval twice when we already have it's throughput.
                // See the eval function for what the rest of the checks are doing:
                if (lobe as *const dyn Lobe != selected_lobe as *const dyn Lobe)
                    && lobe.matches_type(lobe_type)
                    && ((is_reflect && lobe.matches_type(LobeType::REFLECTION))
                        || (!is_reflect && lobe.matches_type(LobeType::TRANSMISSION)))
                {
                    result + lobe.eval(wo, wi)
                } else {
                    result
                }
            })
        } else {
            throughput
        };

        (throughput, world_wi, pdf, sampled_lobe_type)
    }

    // pub fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>], fl: LobeType) -> Spectrum {
    //     self.lobes.iter().fold(Spectrum::black(), |f, &lobe| {
    //         if lobe.matches_type(fl) {
    //             f + lobe.rho_hd(wo, samples)
    //         } else {
    //             f
    //         }
    //     })
    // }

    // pub fn rho_hh(&self, samples0: &[Vec2<f64>], samples1: &[Vec2<f64>], fl: LobeType) -> Spectrum {
    //     self.lobes.iter().fold(Spectrum::black(), |f, &lobe| {
    //         if lobe.matches_type(fl) {
    //             f + lobe.rho_hh(samples0, samples1)
    //         } else {
    //             f
    //         }
    //     })
    // }
}
