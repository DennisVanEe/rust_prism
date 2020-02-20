use crate::integrator::Integrator;
use crate::film::{TileOrdering, Film};
use crate::filter::PixelFilter;

use std::thread::{self, JoinHandle};

// The ThreadData struct has all of the data that is unique to a single thread. So this includes
// its own sampler and whatnot.
pub struct ThreadData<'a, I: Integrator, O: TileOrdering> {
    integrator: I,           // The specific integrator we are dealing with
    filter: &'a PixelFilter, // The filtering algorithm
    film: &'a Film<O>,       // A film tile for the integrator we are dealing with
}

// Specifies the values that are returned:
pub struct RenderThreadReturn {

}

// The threadpool that will be doing all of the rendering work we want it to:
pub struct RenderThreadPool<'a, I: Integrator, O: TileOrdering> {
    threads: Vec<RenderThread<'a, I, O>>,
}

impl<'a, I: Integrator, O: TileOrdering> RenderThreadPool<'a, I, O> {
    pub fn new(num_threads: usize) -> Self {

    }
}

struct RenderThread<'a, I: Integrator, O: TileOrdering>  {
    id: usize,
    thread: JoinHandle<RenderThreadReturn>,
}

// The main render loop function that will handle the rendering.
fn render_loop<'a, I: Integrator, O: TileOrdering>(data: ThreadData<'a, I, O>) {
    loop {
        // First we try to get a specific tile:
        if let Some(tile) = data.film.get_tile() {
            for (&mut pixel, pixel_pos) in tile.iter_mut() {
                
            }
        } else {

        }
    }
}

impl<'a, I: Integrator, O: TileOrdering> RenderThread<'a, I, O> {
    fn new(id: usize, data: ThreadData<'a, I, O>) -> Self {
        let thr = thread::spawn( move || { render_loop(data); });
    }
}