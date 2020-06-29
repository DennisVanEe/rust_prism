use crate::film::TILE_SIZE;
use crate::math::vector::Vec2;
use array_init;
use pmj::Sample;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

// TODO

/// If the path samples more values, then just use regular PCG32 random values.
const NUM_SAMPLES_PER_PATH: usize = 32;
/// Better performance if you turn this off. May otherwise lead to more pleasing samples.
const BLUE_NOISE_RETRY_COUNT: u32 = 0;

pub struct Sampler {
    num_samples_per_pixel: usize,
    pixel_samples: [Vec<Sample>; TILE_SIZE],
    curr_pixel: usize,
    curr_sample: usize,
    rand: Pcg32,
}

impl Sampler {
    pub fn new(num_pixel_samples: u32, backup_seed: u64) -> Self {
        let num_samples_per_pixel = NUM_SAMPLES_PER_PATH * (num_pixel_samples as usize);
        Sampler {
            num_samples_per_pixel,
            pixel_samples: array_init::array_init(|_| Vec::with_capacity(num_samples_per_pixel)),
            curr_pixel: 0,
            curr_sample: 0,
            rand: Pcg32::seed_from_u64(backup_seed),
        }
    }

    /// Prepares sampler for another set of tiles.
    pub fn start_tile(&mut self, seed: u64) {
        let mut rand = Pcg32::seed_from_u64(seed as u64);
        for sample in self.pixel_samples.iter_mut() {
            pmj::generate(
                self.num_samples_per_pixel,
                BLUE_NOISE_RETRY_COUNT,
                &mut rand,
                sample,
            );
        }

        self.curr_pixel = 0;
        self.curr_sample = 0;
    }

    pub fn next_pixel(&mut self) {
        self.curr_pixel += 1;
        self.curr_sample = 0;
    }

    // Retrieves a sample value:
    pub fn sample(&mut self) -> Vec2<f64> {
        // First check if we are in range of the samples:
        let samples = match self.pixel_samples.get(self.curr_pixel) {
            Some(samples) => samples,
            _ => {
                return Vec2 {
                    x: self.rand.gen(),
                    y: self.rand.gen(),
                }
            }
        };

        match samples.get(self.curr_sample) {
            Some(sample) => {
                self.curr_sample += 1;
                Vec2 {
                    x: sample.x() as f64,
                    y: sample.y() as f64,
                }
            }
            _ => Vec2 {
                x: self.rand.gen(),
                y: self.rand.gen(),
            },
        }
    }
}
