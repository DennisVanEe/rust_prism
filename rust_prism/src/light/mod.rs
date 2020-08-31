pub mod area;
pub mod light_picker;
pub mod many_lights;
pub mod point;

use crate::geometry::GeomInteraction;
use crate::sampler::Sampler;
use crate::scene::{GeomRef, Scene};
use crate::shading::lobe::LobeType;
use crate::shading::material::{Bsdf, ShadingCoord};
use crate::spectrum::Color;
use pmath::ray::Ray;
use pmath::sampling;
use pmath::vector::{Vec2, Vec3};

use bitflags::bitflags;

bitflags! {
    pub struct LightType : u32 {
        // Whether or not the light is a delta position light (that is, it's position
        // is a delta function):
        const DELTA_POSITION = 1 << 0;
        // Whether the direction is a delta direction
        const DELTA_DIRECTION = 1 << 1;
        const AREA = 1 << 2;
        const INFINITE = 1 << 3;
    }
}

/// An interface for defining a light in the scene. Lights are transformed into world
/// space when being committed to a scene.
pub trait Light: Sync + 'static {
    /// Samples the light from a specific position (`point`) in world space, a `time` in case the light
    /// varies over time, the `scene` in case it needs it, and a random value (`u`) used to sample the light.
    ///
    /// Returns values in this order:
    /// *`Color`: potential (if no occlusion occurs) energy the light contributes
    /// *`Vec3<f64>`: world space location of where the light will get hit (so one can calculate the wi value themselves)
    /// *`f64`: the probability density for the light sample
    fn sample(
        &self,
        point: Vec3<f64>,
        time: f64,
        scene: &Scene,
        u: Vec2<f64>,
    ) -> (Color, Vec3<f64>, f64);

    /// Given a shading point and direction in world space, returns the pdf.
    fn pdf(&self, shading_point: Vec3<f64>, wi: Vec3<f64>) -> f64;

    /// Returns the total power of the light.
    fn power(&self) -> Color;

    /// Given a `point` on the light and direction (`w`) pointing away from the light, return the color.
    fn eval(&self, point: Vec3<f64>, w: Vec3<f64>) -> Color;

    /// Whether or not the light is a delta (like a point light):
    fn is_delta(&self) -> bool;

    /// Returns the geometry associated with the light (if there is any. Returns `None`
    /// when there isn't any light at all):
    fn get_geom(&self) -> Option<GeomRef>;

    /// Returns the centroid of the light source:
    fn get_centroid(&self) -> Vec3<f64>;
}

/// Samples a light directly using MIS. If there is occlusion, false (and color is black), otherwise
/// it returns true and whatever the color is. This is for hard-surfaces (not mediums).
///
/// # Arguments
/// * `interaction`: World space of the interaction where we are shading from.
/// * `bsdf`: The bsdf at the point we are shading form.
/// * `time`: The time
/// * `sampler`: The sampler used to sample the bsdf and light.
/// * `scene`: The scene used for visibility testing and used by the light if necessary.
/// * `light_id`: The light id of the light we are directly sampling.
/// * `specular`: Whether to handle specular lobes or not.
pub fn estimate_direct_light(
    interaction: GeomInteraction,
    bsdf: &Bsdf,
    time: f64,
    sampler: &mut Sampler,
    scene: &Scene,
    light_id: u32,
    specular: bool,
) -> Color {
    let light = scene.get_light(light_id);
    let lobe_type = if specular {
        LobeType::ALL
    } else {
        let mut removed = LobeType::ALL;
        removed.remove(LobeType::SPECULAR);
        removed
    };

    let shading_coord = ShadingCoord::new(interaction);

    // First we sample the light source:
    let final_color = {
        let (light_color, light_point, light_pdf) =
            light.sample(interaction.p, time, scene, sampler.sample());
        // We don't need to normalize this:
        let wi = light_point - interaction.p;

        // Then we evaluate the bsdf given this light sample:
        if (light_pdf > 0.0) && !light_color.is_black() {
            let bsdf_color = bsdf
                .eval(interaction.wo, wi, lobe_type, shading_coord)
                .scale(wi.dot(interaction.shading_n).abs());
            let bsdf_pdf = bsdf.pdf(interaction.wo, wi, lobe_type, shading_coord);

            if !bsdf_color.is_black() {
                // If the path is unoccluded, we can go ahead and add it's attribute
                if !scene.intersect_test(Ray::new_extent(interaction.p, wi, time, 1.0)) {
                    if light.is_delta() {
                        (bsdf_color * light_color).scale(1.0 / light_pdf)
                    } else {
                        let weight = sampling::power_heuristic(1, light_pdf, 1, bsdf_pdf);
                        (bsdf_color * light_color).scale(weight / light_pdf)
                    }
                } else {
                    Color::black()
                }
            } else {
                Color::black()
            }
        } else {
            Color::black()
        }
    };

    // Then we sample the bsdf:

    // We only sample the bsdf if the light isn't a delta light and has geometry:
    if let Some(light_geom) = light.get_geom() {
        let (bsdf_color, bsdf_wi, bsdf_pdf, sampled_lobe_type) =
            bsdf.sample(interaction.wo, sampler.sample(), lobe_type, shading_coord);
        let bsdf_color = bsdf_color.scale(bsdf_wi.dot(interaction.shading_n).abs());
        let sampled_specular = sampled_lobe_type.contains(LobeType::SPECULAR);

        if !bsdf_color.is_black() && (bsdf_pdf > 0.0) {
            let weight = if !sampled_specular {
                let light_pdf = light.pdf(interaction.p, bsdf_wi);
                if light_pdf == 0.0 {
                    // Nothing more to contribute to the final color, as the bsdf sample didn't:
                    return final_color;
                } else {
                    sampling::power_heuristic(1, bsdf_pdf, 1, light_pdf)
                }
            } else {
                1.0
            };

            // See if our bsdf sample hits the light, and add it's contribution:
            let sample_ray = Ray::new(interaction.p, bsdf_wi, time);
            match scene.intersect(sample_ray) {
                Some(intersected_light_interaction)
                    if intersected_light_interaction.geom == light_geom =>
                {
                    let light_color = light.eval(intersected_light_interaction.p, -bsdf_wi);
                    final_color + (light_color + bsdf_color).scale(weight / bsdf_pdf)
                }
                None => final_color,
            }
        } else {
            final_color
        }
    } else {
        final_color
    }
}
