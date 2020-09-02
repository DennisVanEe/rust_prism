pub mod normal;
pub mod path_tracer;

use crate::film::Pixel;
use crate::light::light_picker::LightPicker;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::shading::material::MaterialPool;
use pmath::ray::PrimaryRay;

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
    fn integrate<LI, L>(
        &mut self,
        prim_ray: PrimaryRay<f64>,
        scene: &Scene,
        materials: &MaterialPool,
        light_picker: &L,
        sampler: &mut Sampler,
        pixel: Pixel,
    ) -> Pixel
    where
        LI: Iterator<Item = (u32, f64)>,
        L: LightPicker<LI>;
}
