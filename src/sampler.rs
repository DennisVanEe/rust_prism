use crate::film::TILE_SIZE;
use crate::math::vector::Vec2;
use pmj::Sample;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

// The sampler here uses "Progressive Multi-Jittered Sample Sequences"
// as described in the paper by: Per Christensen, Andrew Kensler, Charlie Kilpatrick
// and the pseudo-code that was added as well.

// If the path samples more values, then just use regular PCG32 random values
const NUM_SAMPLES_PER_PATH: usize = 32;
// What the original paper had, can try different values..
const BLUE_NOISE_RETRY_COUNT: u32 = 10;

pub struct Sampler {
    num_samples: usize,
    samples: Vec<Sample>,
    path_count: usize,       // The current path we are sampling in this tile
    curr_path_sample: usize, // The current path sample.
    rand: Pcg32,             // Used when we run out of path samples
}

impl Sampler {
    pub fn new(num_pixel_samples: u32) -> Self {
        let num_pixel_samples = num_pixel_samples as usize;
        let num_samples = num_pixel_samples * NUM_SAMPLES_PER_PATH * TILE_SIZE;
        Sampler {
            num_samples,
            samples: Vec::with_capacity(num_samples),
            path_count: 0,
            curr_path_sample: 0,
            rand: Pcg32::from_entropy(),
        }
    }

    // Use this to start a new tile:
    pub fn start_tile(&mut self, tile_id: u32) {
        let mut rand = Pcg32::seed_from_u64(tile_id as u64);
        unsafe {
            // The capactiy should remain the same. This will allow us
            // to regenerate samples without constantly allocating memory:
            self.samples.set_len(0);
        }
        pmj::generate(
            self.num_samples,
            BLUE_NOISE_RETRY_COUNT,
            &mut rand,
            &mut self.samples,
        );
        self.path_count = 0;
        self.curr_path_sample = 0;
    }

    // Increments to the next path:
    pub fn next_path(&mut self) {
        self.curr_path_sample = 0;
        self.path_count += 1;
    }

    // Retrieves a sample value:
    pub fn sample(&mut self) -> Vec2<f64> {
        // If we have gone over, we just generate random samples:
        if self.curr_path_sample >= NUM_SAMPLES_PER_PATH {
            return Vec2 {
                x: self.rand.gen(),
                y: self.rand.gen(),
            };
        }

        let global_sample_index = (self.path_count * NUM_SAMPLES_PER_PATH) + self.curr_path_sample;
        let sample = unsafe { self.samples.get_unchecked(global_sample_index) };
        self.curr_path_sample += 1;

        Vec2 {
            x: sample.x() as f64,
            y: sample.y() as f64,
        }
    }
}
