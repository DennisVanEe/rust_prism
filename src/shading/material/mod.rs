pub mod matte;
pub mod plastic;

use crate::geometry::Interaction;
use crate::math::vector::{Vec2, Vec3};
use crate::memory::allocators::DynStackAlloc;
use crate::shading::lobe::{Lobe, LobeType};
use crate::spectrum::RGBSpectrum;

use arrayvec::ArrayVec;

// Public so that anyone making materials has this information.
// This can be pretty important:
pub const MAX_NUM_LOBES: usize = 8;

#[derive(Clone)]
pub struct Bsdf<'a> {
    // Because we are allocating Lobes using a DynAlllocator,
    // we don't want the Box to drop anything:
    lobes: ArrayVec<[&'a dyn Lobe; MAX_NUM_LOBES]>,
    // The geometric n value:
    pub geometry_n: Vec3<f64>,
    // The shading coordinate space:
    pub n: Vec3<f64>,
    pub s: Vec3<f64>,
    pub t: Vec3<f64>,
    eta: f64,
}

impl<'a> Bsdf<'a> {
    pub fn new(interaction: Interaction, eta: f64, in_lobes: &[&'a dyn Lobe]) -> Self {
        // Check if we can fit that many lobes:
        debug_assert!(in_lobes.len() <= MAX_NUM_LOBES);
        // ArrayVec has no clone_from_slice function, so we do it this way:
        let mut lobes = ArrayVec::<[_; MAX_NUM_LOBES]>::new();
        for &lobe in in_lobes {
            unsafe { lobes.push_unchecked(lobe) };
        }

        let s = interaction.dpdu.normalize();
        Bsdf {
            lobes,
            geometry_n: interaction.n,
            n: interaction.shading_n,
            s,
            t: interaction.shading_n.cross(s),
            eta,
        }
    }

    pub fn new_empty(interaction: Interaction, eta: f64) -> Self {
        let s = interaction.dpdu.normalize();
        Bsdf {
            lobes: ArrayVec::<[_; MAX_NUM_LOBES]>::new(),
            geometry_n: interaction.n,
            n: interaction.shading_n,
            s,
            t: interaction.shading_n.cross(s),
            eta,
        }
    }

    pub fn add_lobe(&mut self, lobe: &'a dyn Lobe) {
        debug_assert!(self.lobes.len() < MAX_NUM_LOBES);
        unsafe { self.lobes.push_unchecked(lobe) };
    }

    // Transforms a world-space vector to shading-space:
    pub fn world_to_shading(&self, v: Vec3<f64>) -> Vec3<f64> {
        Vec3 {
            x: v.dot(self.s),
            y: v.dot(self.t),
            z: v.dot(self.n),
        }
    }

    pub fn shading_to_world(&self, v: Vec3<f64>) -> Vec3<f64> {
        Vec3 {
            x: (self.s.x * v.x) + (self.t.x * v.y) + (self.n.x * v.z),
            y: (self.s.y * v.x) + (self.t.y * v.y) + (self.n.y * v.z),
            z: (self.s.z * v.x) + (self.t.z * v.y) + (self.n.z * v.z),
        }
    }

    pub fn num_has_type(&self, fl: LobeType) -> usize {
        self.lobes.iter().fold(
            0,
            |count, &lobe| {
                if lobe.has_type(fl) {
                    count + 1
                } else {
                    count
                }
            },
        )
    }

    pub fn f(&self, wo: Vec3<f64>, wi: Vec3<f64>, fl: LobeType) -> RGBSpectrum {
        // To prevent light leaking and dark spots, we need to check if wi and wo
        // are on the same side according to the geometric normal:
        let is_reflect = wi.dot(self.geometry_n) * wo.dot(self.geometry_n) > 0.;
        // Transform them to shading space first:
        let wo = self.world_to_shading(wo);
        let wi = self.world_to_shading(wi);

        self.lobes.iter().fold(RGBSpectrum::black(), |f, &lobe| {
            // Make sure that, if it is reflected, then the lobe ONLY has reflection,
            // and if it isn't, then the lobe ONLY has transmission:
            if lobe.has_type(fl)
                && ((is_reflect && lobe.has_type(LobeType::REFLECTION))
                    || (!is_reflect && lobe.has_type(LobeType::TRANSMISSION)))
            {
                f + lobe.f(wo, wi)
            } else {
                f
            }
        })
    }

    pub fn rho_hd(&self, wo: Vec3<f64>, samples: &[Vec2<f64>], fl: LobeType) -> RGBSpectrum {
        self.lobes.iter().fold(RGBSpectrum::black(), |f, &lobe| {
            // Make sure that, if it is reflected, then the lobe ONLY has reflection,
            // and if it isn't, then the lobe ONLY has transmission:
            if lobe.has_type(fl) {
                f + lobe.rho_hd(wo, samples)
            } else {
                f
            }
        })
    }

    pub fn rho_hh(
        &self,
        samples0: &[Vec2<f64>],
        samples1: &[Vec2<f64>],
        fl: LobeType,
    ) -> RGBSpectrum {
        self.lobes.iter().fold(RGBSpectrum::black(), |f, &lobe| {
            // Make sure that, if it is reflected, then the lobe ONLY has reflection,
            // and if it isn't, then the lobe ONLY has transmission:
            if lobe.has_type(fl) {
                f + lobe.rho_hh(samples0, samples1)
            } else {
                f
            }
        })
    }
}

// This defines a way to allocate all of the appropriate materials into a bsdf.
pub trait Material {
    // I'll worry about generating bssrdfs later:
    fn compute_bsdf<'a>(
        &self,
        interaction: Interaction,
        memory: &'a DynStackAlloc,
        use_multiple_lobes: bool,
    ) -> Bsdf<'a>;
}
