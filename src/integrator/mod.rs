pub mod direct_light;

use crate::camera::Camera;
use crate::film::{Film, TileIndex};
use crate::sampler::Sampler;
use crate::scene::Scene;

// Parameters used by the integrator when making requests:
pub struct Request {
    pub sample_arr_sizes_1d: Vec<usize>, // Used by the sampler
    pub sample_arr_sizes_2d: Vec<usize>, // "                 "
}

// Parameters shared by all integrators go here:
pub struct IntegratorParam<'a, S: Sampler, C: Camera> {
    scene: &'a Scene<'a>, // The scene that we are rendering
    sampler: &'a mut S,   // If the integrator needs to prepare any sample arrays
    camera: &'a C,        // The camera we are using for rendering
}

// The parameters that get passed to the integrator everytime
// we render the scene:
pub struct RenderParam<'a, S: Sampler, C: Camera> {
    film: &'a Film,        // The film that is being written to
    tile_index: TileIndex, // The tile index of the film being written to
    scene: &'a Scene<'a>,  // The scene that is being rendered
    sampler: &'a mut S,    // The sampler used to extract samples as it's rendering
}

pub trait Integrator<S: Sampler, C: Camera>: Clone {
    // If there is any extra information NOT shared amonst integrators:
    type Param;

    // Returns a new integrator. We then clone the integrator each time we a thread
    // wants to use it:
    fn new(param: Self::Param, int_param: IntegratorParam<S, C>) -> Self;

    // Given a tile index and the scene, go ahead and fill the film as appropriate:
    fn render(&mut self, param: RenderParam<S, C>) -> TileIndex;
}
