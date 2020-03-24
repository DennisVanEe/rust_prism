use crate::camera::Camera;
use crate::film::{Pixel, Film, TILE_DIM};
use crate::integrator::{Integrator, RenderParam};
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::math::vector::Vec2;

use simple_error::{self, SimpleResult};

use std::thread::{self, JoinHandle};

/// The `RenderThreadPool` is in charge of the threads that will be rendering the scene as. It is also the owner of the resources
/// that are passed into each thread. This includes the `Camera`, `Film`, and `Scene`.
pub struct RenderThreadPool<'a> {
    threads: Vec<RenderThread<'a>>, // All of the threads the renderpool manages (including the thread it manages itself)
    num_threads: usize,
    integrator: Box<dyn Integrator>,
    sampler: Box<dyn Sampler>,
    camera: Box<dyn Camera>,
    film: Film,
    scene: Scene<'a>,
}

impl<'a> RenderThreadPool<'a>
{
    /// Creates a new `RenderThreadPool` with the given number of threads.
    /// 
    /// # Arguments
    /// * `num_threads` - The number of threads to use, including the main thread.
    /// * `integrator` - The integrator that is to be used with rendering.
    /// * `sampler` - The sampler that is to be used with rendering.
    /// * `camera` - The camera that is to be used with rendering.
    /// * `film` - The film that is being rendered to.
    /// * `scene` - The scene that is being rendered.
    /// 
    /// # Panics
    /// Panics if the number of threads is invalid. Note that more threads than the HW supports
    /// concurrently is allowed, just not recommended.
    pub fn new(
        num_threads: usize,
        integrator: Box<dyn Integrator>,
        sampler: Box<dyn Sampler>,
        camera: Box<dyn Camera>,
        film: Film,
        scene: Scene,
    ) -> Self {
        assert_ne!(num_threads, 0);

        // Exclude the main thread:
        let num_threads = -1;
        Ok(RenderThreadPool {
            threads: Vec::with_capacity(num_threads),
            num_threads,
            integrator,
            sampler,
            camera,
            film,
            scene,
        })
    }

    /// Starts the render process. The function doesn't return until all of the rendering was deamed complete.
    pub fn start_render(&mut self) {
        // Go through and create the different threads:
        // ID 0 is reserved for the main thread:

        let camera = self.camera.deref();
        let film = &self.film;
        let scene = &self.scene;

        for id in 1..=self.num_threads {
            let clone_integrator = Box::new(self.integrator.deref().clone());
            let clone_sampler = Box::new(self.sampler.deref().clone());

            self.threads.push(RenderThread::new(
                id,
                clone_integrator,
                clone_sampler,
                camera,
                film,
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
        let sampler = &mut self.sampler;

        // Have the main thread render the tile as well:
        render(
            integrator,
            sampler,
            camera,
            film,
            scene,
            main_init_index,
        );

        // Wait for the other threads to have finished as well:
        for th in self.threads {
            th.join();
        }
    }
}

struct RenderThread<'a> {
    thread: Option<JoinHandle<()>>,

    integrator: Box<dyn Integrator>,  // This needs to be a box because each thread needs to manage one of these themselves.
    sampler: Box<dyn Sampler>,        // Read the above snippet.
    camera: &'a dyn Camera,
    film: &'a Film,
    scene: &'a Scene<'a>,

    id: usize,
}

impl<'a> RenderThread<'a>
{
    fn new(
        id: usize,
        integrator: Box<dyn Integrator>,
        sampler: Box<dyn Sampler>,
        camera: &dyn Camera,
        film: &Film,
        scene: &Scene,
    ) -> Self {
        // Don't spawn the thread yet:
        RenderThread {
            thread: None,
            integrator,
            sampler,
            camera,
            film,
            scene,
            id,
        }
    }

    fn start_render(&mut self) {
        // If the thread was already spawned, panic!
        assert!(
            self.thread.is_none(),
            "Thread with id: {} was already started",
            self.id
        );

        let integrator = &self.integrator;
        let sampler = &mut self.sampler;
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

/// The render function is the function that loops over specified tiles until the film
/// returns `None` for the tiles.
/// 
/// # Arguments
/// * `integrator` - The integrator that is being used to render the scene.
/// * `Sampler` - The sampler that is being used by the integrator.
/// * `Camera` - The camera that is being used to render the scene.
/// * `Film` - The film being rendered to.
/// * `Scene` - The scene being rendered.
fn render(
    integrator: &dyn Integrator,
    sampler: &mut dyn Sampler,
    camera: &dyn Camera,
    film: &Film,
    scene: &Scene,
) {
    loop {
        // We start by getting a tile:
        let mut film_tile = if let Some(film_tile) = film.get_tile() {
            film_tile
        } else {
            break
        };
        let base_pixel_pos = film_tile.pos;

        // Prepare the sampler to start a new pixel tile:
        sampler.start_tile(film_tile.seed);

        for (i, pixel) in film_tile.iter_mut().enumerate() {
            let pixel_pos_delta = Vec2 {
                x: i % TILE_DIM,
                y: i / TILE_DIM,
            };
            sampler.start_pixel(base_pixel_pos + pixel_pos_delta);

            // TODO: prepare other values for the integrator (like
            // camera values and whatnot)

            // We loop until we exhausted all of the pixel samples:
            loop {
                // Prepare the render parameters:
                let param = RenderParam {
                    pixel: pixel.to_render_pixel(),
                    scene,
                    sampler,
                };
                // Render the pixel and set the updated value:
                let result = integrator.render(param);
                pixel = Pixel::from_render_pixel(result);

                // We loop over the same pixel until we have run out of pixel
                // samples.
                if !sampler.next_pixel_sample() {
                    break;
                }
            }
        }
    }
}
