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
// The number of tiles to calculate the pmj values for before hand
const NUM_TILES_PRE_CALC: usize = 8;
// Better performance if you essentially turn this off:
const BLUE_NOISE_RETRY_COUNT: u32 = 1;

pub struct Sampler {
    num_tile_samples: usize,
    num_total_samples: usize,
    samples: Vec<Sample>,
    curr_sample: usize, // The current path sample.
    rand: Pcg32,        // Used when we run out of path samples
}

impl Sampler {
    pub fn new(num_pixel_samples: u32) -> Self {
        let num_pixel_samples = num_pixel_samples as usize;
        let num_tile_samples = num_pixel_samples * NUM_SAMPLES_PER_PATH * TILE_SIZE;
        let num_total_samples = num_tile_samples * NUM_TILES_PRE_CALC;
        Sampler {
            num_tile_samples,
            num_total_samples,
            samples: Vec::with_capacity(num_total_samples),
            curr_sample: 0,
            rand: Pcg32::from_entropy(),
        }
    }

    // Use this to start a new tile:
    pub fn start_tile(&mut self, tile_id: u32) {
        // Check if we still have a 85% of samples left to calculate this tile:
        let remaining_samples = (self.num_total_samples - self.curr_sample) as f64;
        if remaining_samples >= 0.85 * (self.num_tile_samples as f64) {
            return;
        }

        // Otherwise, go ahead and generate more samples:
        let mut rand = Pcg32::seed_from_u64(tile_id as u64);
        unsafe {
            // The capactiy should remain the same. This will allow us
            // to regenerate samples without constantly allocating memory:
            self.samples.set_len(0);
        }
        pmj::generate(
            self.num_tile_samples,
            BLUE_NOISE_RETRY_COUNT,
            &mut rand,
            &mut self.samples,
        );
    }

    // Increments to the next path:
    pub fn next_path(&mut self) {
        // self.curr_path_sample = 0;
        // self.path_count += 1;
    }

    // Retrieves a sample value:
    pub fn sample(&mut self) -> Vec2<f64> {
        // If we have gone over, we just generate random samples:
        if self.curr_sample >= self.num_total_samples {
            return Vec2 {
                x: self.rand.gen(),
                y: self.rand.gen(),
            };
        }

        let sample = unsafe { self.samples.get_unchecked(self.curr_sample) };
        self.curr_sample += 1;

        Vec2 {
            x: sample.x() as f64,
            y: sample.y() as f64,
        }
    }
}
