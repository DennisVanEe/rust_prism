use crate::sampler::Sampler;
use crate::film::Film;

use std::thread::JoinHandle;
use std::sync::atomic::AtomicUsize;

use simple_error::{bail, SimpleResult};

// Adaptive sampling is performed is performed the way that DreamWorks does it:
// ("Global Adaptive Sampling Hierarchies in Production Ray Tracing")
// So, knowing which tile to pick is relatively straight forward (just follow a Morton code):

// A specialized threadpool designed to handle rendering. Create only once
// any other threadpool is closed and cleaned up.
pub struct RenderThreadPool {
    threads: Vec<JoinHandle<T>>,

    // Each RenderThread increments this value to get access
    // to the next tile. The actual tile ordering is defined
    // by the PixelBuffer. Adaptive sampling will just skip
    // any tiles that aren't important.
    curr_tile: AtomicUsize,
}

impl RenderThreadPool {
    pub fn new(num_threads: usize) -> SimpleResult<Self> {
        // First we checked if the number of threads is indeed positive:
        if num_threads == 0 {
            bail!("Can't create thread pool with 0 threads.");
        }

        // Now we can go ahead and spawn each of the threads:
        let threads = Vec::with_capacity(num_threads);
        for _ in 0..num_threads {

        }
    }
}

struct RenderThread<S: Sampler> {
    // Every thread has its own sampler:
    sampler: S,

}

fn render_thread_run() {

}