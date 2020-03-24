use crate::film::RenderPixel;
use crate::sampler::Sampler;
use crate::scene::Scene;

pub mod direct_light;

/// Parameters given to all integrators when calling the integrator's new function.
pub struct PreprocessParam<'a> {
    scene: &'a Scene<'a>,           // The scene that we are rendering.
    sampler: &'a mut dyn Sampler,   // If the integrator needs to prepare any sample arrays, then do so with this.
}

/// Parameters given to the integrator for each pixel being rendered.
pub struct RenderParam<'a> {
    /// THe current value of the pixel being rendered.
    pixel: RenderPixel,
    /// The current scene being rendered.
    scene: &'a Scene<'a>,
    /// The current sampler to use when needing random variables.
    sampler: &'a dyn Sampler,
}

/// The `Integrator` is in charge of rendering an individual pixel at a time. It doesn't have to worry about
/// multi-threading as each resource given to it is either read-only or it has sole ownership of it.
pub trait Integrator: Clone {
    /// Performs any necessary preprocessing for the integrator.
    /// 
    /// # Arguments
    /// * `param` - The provided parameters (see section on `PreprocessParam`).
    fn preprocess(param: PreprocessParam);

    /// Renders a single pixel and returns the updated value of the pixel.
    /// 
    /// # Arguments
    /// * `param` - The provided render parameters (see section on `RenderParam`).
    fn render(&mut self, param: RenderParam) -> RenderPixel;
}
