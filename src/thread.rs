use crate::camera::Camera;
use crate::film::tile_schedular::TileSchedular;
use crate::film::Film;
use crate::film::TileIndex;
use crate::integrator::{Integrator, RenderParam};
use crate::sampler::Sampler;
use crate::scene::Scene;

use simple_error::{self, SimpleResult};

use std::thread::{self, JoinHandle};

pub struct RenderThreadPool<'a, C: Camera, S: Sampler, I: Integrator<S, C>, T: TileSchedular> {
    threads: Vec<RenderThread<'a, C, S, I, T>>, // All of the threads the renderpool manages (including the thread it manages itself)
    num_threads: usize,
    integrator: I, // The RenderThreadPool obtains ownership of both the sampler and the integrator, film and camera
    sampler: S,
    camera: C,
    tile_schedular: T,
    film: Film,
    scene: Scene<'a>,
}

impl<'a, C: Camera, S: Sampler, I: Integrator<S, C>, T: TileSchedular>
    RenderThreadPool<'a, C, S, I, T>
{
    // The number threads is the TOTAL number of threads, including the current "main" thread:
    pub fn new(
        num_threads: usize,
        integrator: I,
        sampler: S,
        camera: C,
        film: Film,
        tile_schedular: T,
        scene: Scene,
    ) -> SimpleResult<Self> {
        if num_threads < 1 {
            simple_error::bail!(
                "The specified number of threads: {} is invalid",
                num_threads
            );
        }

        // Exclude the main thread:
        let num_threads = -1;
        Ok(RenderThreadPool {
            threads: Vec::with_capacity(num_threads),
            num_threads,
            integrator,
            sampler,
            camera,
            tile_schedular,
            film,
            scene,
        })
    }

    // This is a blocking function, that is, this function won't exit until all of the threads are finished with
    // their rendering task:
    pub fn start_render(&mut self) {
        // Go through and create the different threads:
        // ID 0 is reserved for the main thread:

        let camera = &self.camera;
        let film = &self.film;
        let tile_schedular = &self.tile_schedular;
        let scene = &self.scene;

        for id in 1..=self.num_threads {
            let clone_integrator = self.integrator.clone();
            let clone_sampler = self.sampler.clone();

            self.threads.push(RenderThread::new(
                id,
                clone_integrator,
                clone_sampler,
                camera,
                film,
                tile_schedular,
                scene,
            ));
        }

        // Generate a tile for the main thread first:
        let main_init_index = self.tile_schedular.init_index();

        // Now go through and make tiles for everyone else:
        let mut tile_indices = Vec::with_capacity(self.num_threads);
        for _ in 0..self.num_threads {
            tile_indices.push(self.tile_schedular.init_index());
        }

        // Now we can go ahead and start each of the different threads:
        for (th, tile) in self.threads.iter_mut().zip(tile_indices.iter()) {
            th.start_render(tile);
        }

        // Now have the main thread also render the scene:

        let integrator = &self.integrator;
        let sampler = &self.sampler;

        // Have the main thread render the tile as well:
        render(
            integrator,
            sampler,
            camera,
            film,
            tile_schedular,
            scene,
            main_init_index,
        );

        // Wait for the other threads to have finished as well:
        for th in self.threads {
            th.join();
        }
    }
}

// The render thread which is used to store all of the information we would like:
struct RenderThread<'a, C: Camera, S: Sampler, I: Integrator<S, C>, T: TileSchedular> {
    thread: Option<JoinHandle<()>>,

    integrator: I,
    sampler: S,
    camera: &'a C,
    film: &'a Film,
    tile_schedular: &'a T,
    scene: &'a Scene<'a>,

    id: usize,
}

impl<'a, C: Camera, S: Sampler, I: Integrator<S, C>, T: TileSchedular>
    RenderThread<'a, C, S, I, T>
{
    fn new(
        id: usize,
        integrator: I,
        sampler: S,
        camera: &C,
        film: &Film,
        tile_schedular: &T,
        scene: &Scene,
    ) -> Self {
        // Don't spawn the thread yet:
        RenderThread {
            thread: None,
            integrator,
            sampler,
            camera,
            film,
            tile_schedular,
            scene,
            id,
        }
    }

    // Starts the thread for rendering witht the given tile index:
    fn start_render(&mut self, init_index: TileIndex) {
        // If the thread was already spawned, panic!
        assert!(
            self.thread.is_none(),
            "Thread with id: {} was already started",
            self.id
        );

        let integrator = &self.integrator;
        let sampler = &self.sampler;
        let camera = self.camera;
        let film = self.film;
        let tile_schedular = self.tile_schedular;
        let scene = self.scene;

        // Now we can go ahead and spawn the thread:
        self.thread = Some(thread::spawn(render(
            integrator,
            sampler,
            camera,
            film,
            tile_schedular,
            scene,
            init_index,
        )));
    }

    fn join(&self) {
        // Not sure what to do if this fails. So for now we'll just panic:
        // TODO: have the thread maybe return some stats
        if let Some(th) = &self.thread {
            th.join()
                .expect("Could not join thread with id: {}", self.id);
        } else {
            panic!("Thread with id: {} wasn't spawned", self.id);
        }
    }
}

// The function that executed the rendering step:
fn render<C: Camera, S: Sampler, I: Integrator<S, C>, T: TileSchedular>(
    integrator: &I,
    sampler: &S,
    camera: &C,
    film: &Film,
    tile_schedular: &T,
    scene: &Scene,
    init_index: TileIndex,
) {
    // The render loop:
    let mut tile_index = init_index;
    loop {
        // Prepare the render parameters:
        let render_param = RenderParam {
            film,
            tile_index,
            scene,
            sampler,
        };

        let finished_tile_index = integrator.render(render_param);

        // Get the next tile. If wer are done, then we go ahead and exit:
        if let Some(index) = tile_schedular.next_index(finished_tile_index) {
            tile_index = index;
        } else {
            break;
        }
    }
}
