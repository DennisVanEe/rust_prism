pub mod matte;
pub mod plastic;

use crate::geometry::GeomInteraction;
use crate::shading::lobe::{Lobe, LobeType};
use crate::spectrum::Color;
use arrayvec::ArrayVec;
use pmath::vector::{Vec2, Vec3};

/// A MaterialPool holds all of the materials during rendering.
pub struct MaterialPool {
    materials: Vec<Box<dyn Material>>,
}

impl MaterialPool {
    pub fn new() -> Self {
        MaterialPool {
            materials: Vec::new(),
        }
    }

    /// Adds a material to the material pool, returns a material_id.
    pub fn add_material<M: Material>(&mut self, material: M) -> u32 {
        let material_id = self.materials.len() as u32;
        self.materials.push(Box::new(material));
        material_id
    }

    pub unsafe fn get_material(&self, material_id: u32) -> &dyn Material {
        &self.materials[material_id as usize]
    }
}

// TODO: in order to incorporate textutres (where the color of the bsdf is effected),
// I need some way to construct bsdfs without allocating memory. I could have lobes specify
// their own prepare function that materials are aware of (that don't allocate memory), but then
// I would need this lobe's memory to exist for all threads... I don't want to have a material pool
// across all of these threads. Maybe we just add something extra to the lobe? So that it can decide?
// I don't know, I'll figure something out.

/// A material defines how to interact with surfaces when a ray hits it
pub trait Material {
    /// Returns a reference to the bsdf and an interaction if this should be updated.
    /// This may be due to bump mapping, for instance.
    fn bsdf(&self, interaction: GeomInteraction) -> (&Bsdf, GeomInteraction);
}

/// Used to convert to and from shading coordinate space:
#[derive(Clone, Copy, Debug)]
pub struct ShadingCoord {
    geometry_n: Vec3<f64>,
    n: Vec3<f64>,
    s: Vec3<f64>,
    t: Vec3<f64>,
}

impl ShadingCoord {
    /// Given an interaction, can construct a new shading coordinate system
    pub fn new(interaction: GeomInteraction) -> Self {
        let s = interaction.dpdu.normalize();
        ShadingCoord {
            geometry_n: interaction.n,
            n: interaction.shading_n,
            s,
            t: interaction.shading_n.cross(s),
        }
    }

    /// Transforms a vector from world space to shading space.
    pub fn world_to_shading_vec(self, v: Vec3<f64>) -> Vec3<f64> {
        Vec3 {
            x: v.dot(self.s),
            y: v.dot(self.t),
            z: v.dot(self.n),
        }
    }

    /// Transforms a vector from shading space to world space.
    pub fn shading_to_world_vec(self, v: Vec3<f64>) -> Vec3<f64> {
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

/// The maximum number of lobes per bsdf.
pub const MAX_NUM_LOBES: usize = 8;

#[derive(Clone)]
pub struct Bsdf {
    lobes: ArrayVec<[Box<dyn Lobe>; MAX_NUM_LOBES]>,
    eta: f64,
}

impl Bsdf {
    /// Creates a new bsdf for opaque materials.
    pub fn new_opaque() -> Self {
        Bsdf {
            lobes: ArrayVec::new(),
            eta: 1.0,
        }
    }

    /// Creates a new bsdf with a given refractive index (`eta`).
    pub fn new(eta: f64) -> Self {
        Bsdf {
            lobes: ArrayVec::new(),
            eta,
        }
    }

    /// Adds a lobe to the Bsdf. If it exceed `MAX_NUM_LOBES`, the function will panic.
    pub fn add_lobe<L: Lobe>(&mut self, lobe: L) {
        self.lobes.push(Box::new(lobe));
    }

    /// Returns the number of lobes that have the specified lobe type:
    pub fn num_contains_type(&self, lobe_type: LobeType) -> usize {
        self.lobes.iter().fold(0, |count, lobe| {
            if lobe.contains_type(lobe_type) {
                count + 1
            } else {
                count
            }
        })
    }

    /// Evaluate the lobe, with `wo` and `wi` in world space.
    pub fn eval(
        &self,
        wo: Vec3<f64>,
        wi: Vec3<f64>,
        lobe_type: LobeType,
        shading_coord: ShadingCoord,
    ) -> Color {
        let shading_wo = shading_coord.world_to_shading_vec(wo);
        let shading_wi = shading_coord.world_to_shading_vec(wi);
        let is_reflect = shading_coord.geometry_n.dot(wo) * shading_coord.geometry_n.dot(wi) > 0.0;

        self.lobes
            .iter()
            .fold(Color::black(), |result_color, lobe| {
                let matches = lobe.contains_type(lobe_type);
                // Checks that, if it's reflected then we have a reflection lobe and if it's
                // not reflected we have a transmission lobe.
                let valid_direction = (is_reflect && lobe.contains_type(LobeType::REFLECTION))
                    || (!is_reflect && lobe.contains_type(LobeType::TRANSMISSION));

                if matches && valid_direction {
                    result_color + lobe.eval(shading_wo, shading_wi)
                } else {
                    result_color // otherwise we do nothing
                }
            })
    }

    /// Evaluate the lobe, with `wo` and `wi` in world space.
    pub fn pdf(
        &self,
        wo: Vec3<f64>,
        wi: Vec3<f64>,
        lobe_type: LobeType,
        shading_coord: ShadingCoord,
    ) -> f64 {
        let shading_wo = shading_coord.world_to_shading_vec(wo);
        let shading_wi = shading_coord.world_to_shading_vec(wi);

        // We are essentially averaging the pdfs that match the flags:
        let (pdf, num_has_type) = self
            .lobes
            .iter()
            .fold((0.0, 0u32), |(pdf_sum, count), lobe| {
                if lobe.contains_type(lobe_type) {
                    (pdf_sum + lobe.pdf(shading_wo, shading_wi), count + 1)
                } else {
                    (pdf_sum, count)
                }
            });
        pdf / (num_has_type as f64)
    }

    /// Samples the bsdf given a `wo` in world space.
    /// Returns, in the following order: resulting throughput, wi (world space), pdf, lobe type of lobe samples:
    pub fn sample(
        &self,
        wo: Vec3<f64>,
        u: Vec2<f64>,
        lobe_type: LobeType,
        shading_coord: ShadingCoord,
    ) -> (Color, Vec3<f64>, f64, LobeType) {
        // First, make sure we only consider lobes that match with the specified LobeType.
        let mut potential_lobes: ArrayVec<[_; MAX_NUM_LOBES]> = ArrayVec::new();
        for &lobe in &self.lobes {
            if lobe.contains_type(lobe_type) {
                potential_lobes.push(&*lobe);
            }
        }
        let num_has_type = potential_lobes.len();
        if num_has_type == 0 {
            return (Color::black(), Vec3::zero(), 0.0, LobeType::NONE);
        }

        // TODO: pick a wiser selection algorithm for lobes.
        let selected_lobe_index = ((u.x * (num_has_type as f64)) as usize).min(num_has_type - 1);
        let selected_lobe = potential_lobes[selected_lobe_index];

        // We still want to use u.x, so we have to remap it so that u can still
        // be between 0 and 1.
        let u = Vec2 {
            x: u.x * (num_has_type - selected_lobe_index) as f64,
            y: u.y,
        };

        // Sample the selected lobe for the wi value:
        let shading_wo = shading_coord.world_to_shading_vec(wo);
        let sampled_lobe_type = selected_lobe.get_type();
        let (selected_color, shading_wi, selected_pdf) = selected_lobe.sample(shading_wo, u);
        let wi = shading_coord.shading_to_world_vec(shading_wi);

        // Take into account all of the other pdf values unless it's specular, then the pdf is 1.
        let pdf = if !sampled_lobe_type.contains(LobeType::SPECULAR) {
            potential_lobes
                .iter()
                .enumerate()
                .fold(selected_pdf, |pdf_sum, (index, lobe)| {
                    if index == selected_lobe_index {
                        pdf_sum
                    } else {
                        pdf_sum + lobe.pdf(wo, wi)
                    }
                })
                / (num_has_type as f64) // Averaging, remember?
        } else {
            selected_pdf
        };

        // Now we calculate the throughput by summing the contributions from each of the lobes.
        let color = if !sampled_lobe_type.contains(LobeType::SPECULAR) {
            // Check if they are on the same side relative to the normal (reflected):
            let is_reflect = wi.dot(self.geometry_n) * wo.dot(self.geometry_n) > 0.;
            potential_lobes
                .iter()
                .enumerate()
                .fold(selected_color, |color, (index, lobe)| {
                    if (selected_lobe_index != index)
                        && ((is_reflect && lobe.contains_type(LobeType::REFLECTION))
                            || (!is_reflect && lobe.contains_type(LobeType::TRANSMISSION)))
                    {
                        selected_color + lobe.eval(shading_wo, shading_wi)
                    } else {
                        selected_color
                    }
                })
        } else {
            selected_color
        };

        (color, wi, pdf, sampled_lobe_type)
    }
}
