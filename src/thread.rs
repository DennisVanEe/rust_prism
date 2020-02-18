use crate::integrator::Integrator;
use crate::film::Film;
use crate::pixel_buffer::TileOrdering;

use std::thread::{self, JoinHandle};

// The ThreadData struct has all of the data that is unique to a single thread. So this includes
// its own sampler and whatnot.
pub struct ThreadData<'a, I: Integrator, O: TileOrdering> {
    integrator: I,     // The specific integrator we are dealing with
    film: &'a Film<O>, // A film tile for the integrator we are dealing with
}

// Specifies any return information that we may want. Not sure what we would want
// to return yet, but it's here in case we want it:
pub struct RenderThreadReturn {

}

// The threadpool that will be doing all of the rendering work we want it to:
pub struct RenderThreadPool {
    threads: Vec<RenderThread>,
}

impl RenderThreadPool {
    pub fn new(num_threads: usize) -> Self {

    }
}

struct RenderThread<'a, I: Integrator, O: TileOrdering>  {
    id: usize,
    thread: JoinHandle<RenderThreadReturn>,
}

impl<'a, I: Integrator, O: TileOrdering> RenderThread<'a, I, O> {
    fn new(id: usize, data: ThreadData<'a, I, O>) -> Self {

        let thread = thread::spawn(move || {
            // First thing we have to do is get a specific tile:
            
        });
    }
}