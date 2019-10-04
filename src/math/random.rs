// This file contains the random number generator
// used throughout the program:

#[derive(Clone, Copy)]
pub struct RandGen {
    inc: u64,
    state: u64,
}

impl RandGen {
    const ONE_MINUS_EPS: f64 = 0.99999999999999989;
    const PCG_32_MULT: u64 = 0x5851f42d4c957f2d;
    const PCG_32_DEFAULT_STATE: u64 = 0x853c49e6748fea9b;
    const PCG_32_DEFAULT_STREAM: u64 = 0xda3e39cb94b95bdb;

    // Creates a default random number generator:
    pub fn new_default() -> Self {
        RandGen {
            inc: Self::PCG_32_DEFAULT_STREAM,
            state: Self::PCG_32_DEFAULT_STATE,
        }
    }

    // Creates a new random number generator given a sequence:
    pub fn new(init_seq: u64) -> Self {
        let mut rng = RandGen {
            inc: (init_seq << 1) | 1,
            state: 0,
        };
        rng.uniform_u32();
        rng.state += Self::PCG_32_DEFAULT_STATE;
        rng.uniform_u32();
        rng
    }

    // Returns a number from 0 to u32::MAX
    pub fn uniform_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(Self::PCG_32_MULT)
            .wrapping_add(self.inc);
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        (xor_shifted >> rot) | (xor_shifted << ((!rot).wrapping_add(1) & 31))
    }

    pub fn uniform_u32_limit(&mut self, limit: u32) -> u32 {
        let threshold = (!limit + 1) % limit;
        loop {
            let r = self.uniform_u32();
            if r >= threshold {
                return r % threshold;
            }
        }
    }

    // Returns a number from 0 to 1
    pub fn uniform_f64(&mut self) -> f64 {
        Self::ONE_MINUS_EPS.min(self.uniform_u32() as f64 * 2.3283064365386963e-10)
    }
}
