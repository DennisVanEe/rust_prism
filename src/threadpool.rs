use std::thread::JoinHandle;

use simple_error::{bail, SimpleResult};

pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> SimpleResult<Self> {
        // First we checked if the number of threads is indeed positive:
        if num_threads == 0 {
            bail!("Can't create thread pool with 0 threads.");
        }
    }

    pub fn execute<F
}