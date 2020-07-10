use crate::film::TILE_SIZE;
use crate::math::vector::Vec2;
use pmj::{self, Sample};
use rand::SeedableRng;
use rand_pcg::Pcg32;

pub struct Sampler<'a> {
    pattern: u32, // The "pattern" is basically the pixel that the sample is being drawn for
    sample: u32,  // The sample is the index of the current sample for a specific pixel
    tables: &'a SampleTables, // All of the samples belong to this
}

impl<'a> Sampler<'a> {
    pub fn new(tables: &'a SampleTables) -> Self {
        Sampler {
            pattern: 0,
            sample: 0,
            tables,
        }
    }

    pub fn sample(&mut self) -> Vec2<f64> {
        let res = self.tables.sample(self.pattern, self.sample);
        self.sample += 1;
        res
    }

    // Need to call when going to the next pixel
    pub fn next_pixel(&mut self) {
        self.pattern += 1;
        self.sample = 0;
    }

    // Need to call when going to next tile
    pub fn start_tile(&mut self, tile_index: u32) {
        self.pattern = tile_index * (TILE_SIZE as u32);
        self.sample = 0;
    }
}

/// The `Sampler` is no longer bound to each thread. Instead, each thread will receive a reference to a single
/// "global" `Sampler` as it no longer generates samples on the go (insstead, each sample is gathered through
/// a table lookup)
pub struct SampleTables {
    samples: Vec<Sample>,
}

// These values are taken from RenderMan's RixRNG implementation.
const NUM_TABLES: usize = 401;
const NUM_SAMPLES_PER_TABLE: usize = 1024;

impl SampleTables {
    /// Generates a new `Sampler`. The `init_seed` parameter is used to create the PMJ sample tables.
    /// The `backup_seed` parameter is used to seed the backup random number generator if we run out of
    /// table entries. The `blue_noise_retry_count` specifies how often to retry for blue noise (set to 0 if you don't
    /// want blue noise).
    pub fn new(init_seed: u64, blue_noise_retry_count: u32) -> Self {
        let mut samples = Vec::with_capacity(NUM_TABLES * NUM_SAMPLES_PER_TABLE);
        for i in 0..NUM_TABLES {
            let mut rand = Pcg32::seed_from_u64(init_seed + (i as u64));
            let mut table = pmj::generate(NUM_SAMPLES_PER_TABLE, blue_noise_retry_count, &mut rand);
            samples.append(&mut table);
        }
        SampleTables { samples }
    }

    fn sample(&self, pattern: u32, sample: u32) -> Vec2<f64> {
        const TOTAL_NUM_SAMPLES: usize = NUM_TABLES * NUM_SAMPLES_PER_TABLE;

        // We ran out of samples:
        if sample > (TOTAL_NUM_SAMPLES as u32) {
            return Vec2 {
                x: Self::hash_to_random_f32(sample, pattern * 0x51633e2d) as f64,
                y: Self::hash_to_random_f32(sample, pattern * 0x68bc21eb) as f64,
            };
        }

        // This code is basically a translation of RenderMan's RixRNGProgressive:

        let pattern1 = Self::hash_to_random_u32(pattern, 0x51633e2d);
        let pattern2 = Self::hash_to_random_u32(pattern, 0x68bc21eb);
        let pattern3 = Self::hash_to_random_u32(pattern, 0x02e5be93);
        let pattern4 = Self::hash_to_random_u32(pattern, 0x967a889b);

        // Select the table:
        let t = pattern1 % (NUM_TABLES as u32);

        // Select a table entry:
        let s = {
            let mut s = sample + t * (NUM_SAMPLES_PER_TABLE as u32);
            if (pattern2 & 0x1) != 0 {
                s = if (sample & 0x1) != 0 {
                    s.wrapping_sub(1)
                } else {
                    s.wrapping_add(1)
                };
            }
            if (pattern2 & 0x2) != 0 {
                s = if (sample & 0x2) != 0 {
                    s.wrapping_sub(2)
                } else {
                    s.wrapping_add(2)
                };
            }
            if (pattern2 & 0x4) != 0 {
                s = if (sample & 0x4) != 0 {
                    s.wrapping_sub(4)
                } else {
                    s.wrapping_add(4)
                };
            }
            if (pattern2 & 0x8) != 0 {
                s = if (sample & 0x8) != 0 {
                    s.wrapping_sub(8)
                } else {
                    s.wrapping_add(8)
                };
            }
            s % (TOTAL_NUM_SAMPLES as u32)
        };

        // Get the samble (and scramble matissa):
        let result = {
            let result = self.samples[s as usize];
            Vec2 {
                x: Self::scramble_f32(result.x(), pattern3),
                y: Self::scramble_f32(result.y(), pattern4),
            }
        };

        // Make sure x and y are in [0, 1)
        Vec2 {
            x: if result.x > 0.999990 {
                0.999990
            } else {
                result.x
            } as f64,
            y: if result.y > 0.999990 {
                0.999990
            } else {
                result.y
            } as f64,
        }
    }

    // Similar to HashToRandomUInt from Pixar's implementation
    fn hash_to_random_u32(value: u32, scramble: u32) -> u32 {
        let result = value ^ scramble;
        let result = result ^ (result >> 17);
        let result = result ^ (result >> 10);
        let result = result.wrapping_mul(0xb36534e5);
        let result = result ^ (result >> 12);
        let result = result ^ (result >> 21);
        let result = result.wrapping_mul(0x93fc4795);
        let result = result.wrapping_mul(0xdf6e307f);
        let result = result ^ (result >> 17);
        let result = result.wrapping_mul(1 | (scramble >> 18));
        result
    }

    // Similar to HashToRandom from Pixar's implementation
    fn hash_to_random_f32(value: u32, scramble: u32) -> f32 {
        let randu32 = Self::hash_to_random_u32(value, scramble);
        (randu32 as f32) / 4298115584.0
    }

    fn scramble_f32(f: f32, scramble: u32) -> f32 {
        // Map to [1, 2)
        let f = f + 1.0;
        // Scramble mantissa (just xor it)
        let i = f.to_bits() ^ (scramble >> 9);
        // Map to [0, 1)
        f32::from_bits(i) - 1.0
    }
}
