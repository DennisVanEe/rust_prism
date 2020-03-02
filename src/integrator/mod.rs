pub mod direct_light;

use crate::scene::Scene;
use crate::sampler::Sampler;
use crate::film::{Film, TileIndex};
use crate::camera::Camera;

// Any preprocess information that may be required for the current thread that
// is rendering:
pub struct IntegratorParam<'a> {
    scene: &'a Scene<'a>,
    // The number of pixel samples:
    num_pixel_samples: usize,
    // The number of dimensions:
    num_dim: usize,
    // The maximum depth of a path:
    max_depth: u32,
    // The camera that is being used for rendering:
    camera: &'a Camera,
}

// The parameters that get passed to the integrator:
pub struct RenderParam<'a> {
    film: &'a Film,
    tile_index: TileIndex,
    scene: &'a Scene<'a>,
}

pub trait Integrator {
    // If there is any extra information NOT shared amonst integrators:
    type Param;

    // Given the scene, perform any preprocessing steps that may be required and create self:
    fn new(param: Self::Param, int_param: IntegratorParam) -> Self;

    // Given a tile index and the scene, go ahead and fill the film as appropriate:
    fn render(&mut self, param: RenderParam);
}
