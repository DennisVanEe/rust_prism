use crate::film::Pixel;
use crate::light::Light;
use crate::math::numbers::Float;
use crate::math::ray::PrimaryRay;
use crate::math::ray::Ray;
use crate::math::vector::{Vec2, Vec3};
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;

/// An `IntegratorManager` is used to spawn integrators for each thread and maintain any
/// information that integrators across different threads may want to use. It is gauranteed
/// that the IntegratorManager instance will exist until all threads have finished rendering.
pub trait IntegratorManager<I: Integrator>: Sync {
    /// Any parameters to pass to the integrator when integrating the scene
    type InitParam;

    /// Creates a new IntegratorManager with the given parameters.
    fn new(param: Self::InitParam) -> Self;
    /// Spawns an integrator for a specific thread with the provided id.
    fn spawn_integrator(&self, thread_id: u32) -> I;
}

/// Defines different integrators for use with PRISM. Each thread gets its own `Integrator` instance.
pub trait Integrator {
    /// Given the primary ray (as a result of the camera), the scene, the sampler, and the
    /// pixel value already present at the point, integrates the specific pixel and returns
    /// the pixel value at the specified location.
    fn integrate(
        &self,
        prim_ray: PrimaryRay<f64>,
        scene: &Scene,
        sampler: &mut Sampler,
        pixel: Pixel,
    ) -> Pixel;
}

/// Given a `point` in world space and a light to sample, performs MIS to directly
/// sample the specified light.
pub fn estimate_direct_light(
    point: Vec3<f64>,      // The point in world space
    time: f64,             // The time to sample this light
    u_bsdf: Vec2<f64>,     // Random used to sample the bsdf
    u_light: Vec2<f64>,    // Random used to sample the light
    light: &dyn Light,     // The light in question
    scene: &Scene,         // The scene in question
    sampler: &mut Sampler, // A sampler for further use
    specular: bool,        // Whether to handle specular components or not
) -> Spectrum {
    // First, sample the light:

    let (li, light_point, light_pdf) = light.sample(point, time, u_light);
    if light_pdf > 0.0 && !li.is_black() {
        let wo = point - light_point;
        // Now check if the path to the light is occluded or not.
        let occl_ray = Ray {
            org: point,
            dir: wo,
            time,
            t_far: 1.0 - f64::SELF_INT_COMP, // This means we don't self intersect
            t_near: f64::SELF_INT_COMP,
        };
        
        // Now sample the bsdf using the sample from the light:
        let bsdf_f = 
    }

    todo!();
}
