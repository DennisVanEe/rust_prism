use crate::integrator::Integrator;
use crate::film::tile_schedular::TileSchedular;
use crate::film::TileIndex;
use crate::filter::PixelFilter;

use std::thread::{self, JoinHandle};

pub struct RenderThreadPool<'a, I: Integrator> {
    threads: Vec<RenderThread<'a, I>>,
    tile_scheduler: 
}

impl<'a, I: Integrator, O: TileOrdering> RenderThreadPool<'a, I, O> {
    pub fn new(num_threads: usize) -> Self {

    }
}

// The render thread which is used to store all of the information we would like:
struct RenderThread<'a, I: Integrator>  {
    // The actual thread:
    thread: JoinHandle<()>,

    // The integrator for the thread:


    // Specifies the ID of the current thread running:
    id: usize,
}

impl<'a, I: Integrator, O: TileOrdering> RenderThread<'a, I, O> {
    fn new(id: usize, data: ThreadData<'a, I, O>) -> Self {
        let thr = thread::spawn( move || {
            if let Some(tile) = data.film.get_tile() {

            } else {
                
            }

            data.integrator.render(, scene: &Scene)
        });
    }
}