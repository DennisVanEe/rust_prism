use crate::camera::{Camera, CameraSample};
use crate::film::{Film, TILE_DIM};
use crate::filter::PixelFilter;
use crate::integrator::{Integrator, IntegratorManager};
use crate::sampler::{SampleTables, Sampler};
use crate::scene::Scene;
use core_affinity;
use crossbeam::thread;
use pmath::vector::Vec2;
use simple_error::{bail, SimpleResult};

/// Basic parameters used independent of the integrator used.
#[derive(Clone, Copy, Debug)]
pub struct RenderParam {
    /// The number of samples to perform for each pixel
    pub num_pixel_samples: u32,
    /// The number of threads to render with
    pub num_threads: u32,
    /// The seed to use when generating sample tables
    pub sample_seed: u64,
    /// The number of attempts when ensuring blue noise in the sampler
    pub blue_noise_count: u32,
    /// Resolution:
    pub res: Vec2<usize>,
}

pub fn render<I: Integrator, M: IntegratorManager<I>>(
    camera: &dyn Camera,
    filter: PixelFilter,
    scene: &Scene,
    param: RenderParam,
    int_param: M::InitParam,
) -> SimpleResult<Film> {
    //
    // Generate the film:
    //

    let res = Vec2 {
        x: param.res.x / TILE_DIM,
        y: param.res.y / TILE_DIM,
    };
    let film = Film::new_zero(res);
    let film_ref = &film;

    //
    // Generate the SampleTables:
    //

    let sample_tables = SampleTables::new(param.sample_seed, param.blue_noise_count);
    let sample_tables_ref = &sample_tables;

    //
    // Get available hardware threads:
    //

    // Check if we will go ahead and bind threads (that is, if we can or not):
    let (bind_threads, core_ids) = match core_affinity::get_core_ids() {
        Some(ids) => {
            // If there are fewer cores than threads demanded, than don't bother binding threads:
            if ids.len() < param.num_threads as usize {
                (false, Vec::new())
            } else {
                (true, ids)
            }
        }
        _ => (false, Vec::new()),
    };
    let core_ids_ref = &core_ids;

    //
    // Create the IntegratorManager:
    //

    let integrator_manager = M::new(int_param);
    let integrator_manager_ref = &integrator_manager;

    // If we're only rendering one thing.
    if param.num_threads <= 1 {
        // Bind the main thread:
        if bind_threads {
            let curr_core_id = core_ids_ref[0];
            core_affinity::set_for_current(curr_core_id);
        }

        let integrator = integrator_manager_ref.spawn_integrator(0);
        let sampler = Sampler::new(sample_tables_ref);
        thread_render(
            0,
            camera,
            filter,
            sampler,
            film_ref,
            scene,
            param.num_pixel_samples,
            integrator,
        );
        return Ok(film);
    }

    // We subtract one because don't want to include the main thread:
    let num_threads = param.num_threads - 1;

    // Launch a bunch of scoped threads:
    let render_result = thread::scope(move |s| {
        // Bind the main thread:
        if bind_threads {
            let curr_core_id = core_ids_ref[0];
            core_affinity::set_for_current(curr_core_id);
        }

        for id in 1..=num_threads {
            s.spawn(move |_| {
                // Bind the threads as appropriate:
                if bind_threads {
                    let curr_core_id = core_ids_ref[id as usize];
                    core_affinity::set_for_current(curr_core_id);
                }

                let integrator = integrator_manager_ref.spawn_integrator(id);
                let sampler = Sampler::new(sample_tables_ref);
                thread_render(
                    id,
                    camera,
                    filter,
                    sampler,
                    film_ref,
                    scene,
                    param.num_pixel_samples,
                    integrator,
                );
            });
        }

        // The "main" thread always had id 0:
        let integrator = integrator_manager_ref.spawn_integrator(0);
        let sampler = Sampler::new(sample_tables_ref);
        thread_render(
            0,
            camera,
            filter,
            sampler,
            film_ref,
            scene,
            param.num_pixel_samples,
            integrator,
        );
    });

    match render_result {
        Ok(_) => Ok(film),
        _ => bail!("Error when executing render threads"),
    }
}

/// The render function is the function that loops over specified tiles until the film
/// returns `None` for the tiles.
///
/// # Arguments
/// * `id` - The id of the current thread.
/// * `camera` - The camera that is being used to render the scene.
/// * `filter` - The filter used when sampling points on the film.
/// * `sampler` - The sampler that is being used by the integrator.
/// * `film` - The film being rendered to.
/// * `scene` - The scene being rendered.
/// * `num_pixel_samples` - The number of samples to perform per pixel
/// * `integrator` - The integrator to be used by this specific thread
fn thread_render<I: Integrator>(
    _id: u32,
    camera: &dyn Camera,
    filter: PixelFilter,
    mut sampler: Sampler,
    film: &Film,
    scene: &Scene,
    num_pixel_samples: u32,
    mut integrator: I,
) {
    loop {
        // When getting the next tile, we also check if any tiles are left in this pass.
        let mut film_tile = match film.get_tile() {
            Some(film_tile) => film_tile,
            _ => break,
        };

        sampler.start_tile(film_tile.index as u32);

        for (i, pixel) in film_tile.data.iter_mut().enumerate() {
            // Make sure we are able to retrieve the next pixel position:
            let pixel_pos = Vec2 {
                x: (film_tile.pos.x + (i % TILE_DIM)) as f64 + 0.5,
                y: (film_tile.pos.y + (i / TILE_DIM)) as f64 + 0.5,
            };

            // Loop over all of the paths:
            for _ in 0..num_pixel_samples {
                // Generate a camera ray:
                let camera_sample = CameraSample {
                    p_film: pixel_pos + filter.sample_pos(sampler.sample()),
                    p_lens: sampler.sample(),
                    time: sampler.sample().x,
                };
                let prim_ray = camera.gen_primary_ray(camera_sample);

                // Now go ahead and integrate for this ray:
                *pixel = integrator.integrate(prim_ray, scene, &mut sampler, *pixel);
            }

            // Tell the samapler we're moving onto the next pixel:
            sampler.next_pixel();
        }

        film.set_tile(film_tile);
    }
}
