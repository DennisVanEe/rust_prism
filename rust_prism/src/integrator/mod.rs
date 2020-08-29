pub mod normal;

use crate::film::Pixel;
use crate::light::Light;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Color;
use pmath::numbers::Float;
use pmath::ray::PrimaryRay;
use pmath::ray::Ray;
use pmath::vector::{Vec2, Vec3};

/// An `IntegratorManager` is used to spawn integrators for each thread and maintain any
/// information that integrators across different threads may want to use. It is guaranteed
/// that the IntegratorManager instance will exist until all threads have finished rendering.
pub trait IntegratorManager<I: Integrator>: Sync {
    /// Spawns an integrator for a specific thread with the provided id.
    fn spawn_integrator(&self, thread_id: u32) -> I;
}

/// Defines different integrators for use with PRISM. Each thread gets its own `Integrator` instance.
pub trait Integrator {
    /// Given the primary ray (as a result of the camera), the scene, the sampler, and the
    /// pixel value already present at the point, integrates the specific pixel and returns
    /// the pixel value at the specified location.
    fn integrate(
        &mut self,
        prim_ray: PrimaryRay<f64>,
        scene: &Scene,
        sampler: &mut Sampler,
        pixel: Pixel,
    ) -> Pixel;
}
