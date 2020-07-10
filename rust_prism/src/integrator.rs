use crate::film::Pixel;
use crate::math::ray::PrimaryRay;
use crate::sampler::Sampler;
use crate::scene::Scene;

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
