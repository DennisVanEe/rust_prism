use crate::math::numbers::Float;
use crate::math::random::RandGen;
use crate::math::util::next_pow2_u64;
use crate::math::vector::Vec2;
use crate::mem;
use crate::sampler::{shuffle, Sampler};

pub struct ZeroTwo {
    num_pixel_samples: usize,
    num_dim: usize,

    // The samples values are stored contiguously in memory but represents
    // a 2D array of values.
    // They are represented as such (first index which dimension, then which pixel sample we care about):
    //
    // dim0: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // dim1: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // ...
    samples_1d: Vec<f64>,
    samples_2d: Vec<Vec2<f64>>,

    // Someone can potentially request arrays of samples. If they do, they will be stored
    // here. The first entry of the tuple stores the length of the array. This is usefull
    // information to have.
    // Each sample gets its own array (so, the length of the vectors is length of array * num_pixel_samples)
    arr_samples_1d: Vec<(usize, Vec<f64>)>,
    arr_samples_2d: Vec<(usize, Vec<Vec2<f64>>)>,

    // State information regarding the sampler. As each thread gets its
    // own sampler, we aren't concerned with race conditions or anything:

    // The current pixel sample we are on:
    index_pixel_sample: usize,
    // The next array we will return when requested (for 1D):
    index_arr_1d: usize,
    // The next array we will return when requested (for 2D):
    index_arr_2d: usize,
    // The current 1d sample index:
    index_1d: usize,
    // The current 2d sample index:
    index_2d: usize,

    rng: RandGen,
}

impl ZeroTwo {
    fn gen_1d_samples(
        &mut self,
        // The number of samples per pixel sample:
        num_pixel_sample_samples: usize,
        // Where we are storing the resulting samples:
        samples: &mut [f64],
    ) {
        // Van Der Corput sequence:
        const VAN_DER_CORPUT: [u32; 32] = [
            0x80000000, 0x40000000, 0x20000000, 0x10000000, 0x8000000, 0x4000000, 0x2000000,
            0x1000000, 0x800000, 0x400000, 0x200000, 0x100000, 0x80000, 0x40000, 0x20000, 0x10000,
            0x8000, 0x4000, 0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100, 0x80, 0x40, 0x20, 0x10,
            0x8, 0x4, 0x2, 0x1,
        ];
        // Make sure we have enough room:
        assert!(samples.len() == num_pixel_sample_samples * self.num_pixel_samples);

        let scramble = self.rng.uniform_u32();
        greycode_sample_1d(&VAN_DER_CORPUT, scramble, samples);
        // Now we should go ahead and shuffle the values:
        for i in 0..self.num_pixel_samples {
            let start_index = i * num_pixel_sample_samples;
            let end_index = start_index + num_pixel_sample_samples;
            let sub_samples = &mut samples[start_index..end_index];
            shuffle(sub_samples, 1, &mut self.rng);
        }
        shuffle(samples, num_pixel_sample_samples, &mut self.rng);
    }

    fn gen_2d_samples(
        &mut self,
        // The number of samples per pixel sample:
        num_pixel_sample_samples: usize,
        // Where we are storing the resulting samples:
        samples: &mut [Vec2<f64>],
    ) {
        // Van Der Corput sequence:
        const SOBOL: [[u32; 32]; 2] = [
            [
                0x80000000, 0x40000000, 0x20000000, 0x10000000, 0x8000000, 0x4000000, 0x2000000,
                0x1000000, 0x800000, 0x400000, 0x200000, 0x100000, 0x80000, 0x40000, 0x20000,
                0x10000, 0x8000, 0x4000, 0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100, 0x80, 0x40,
                0x20, 0x10, 0x8, 0x4, 0x2, 0x1,
            ],
            [
                0x80000000, 0xc0000000, 0xa0000000, 0xf0000000, 0x88000000, 0xcc000000, 0xaa000000,
                0xff000000, 0x80800000, 0xc0c00000, 0xa0a00000, 0xf0f00000, 0x88880000, 0xcccc0000,
                0xaaaa0000, 0xffff0000, 0x80008000, 0xc000c000, 0xa000a000, 0xf000f000, 0x88008800,
                0xcc00cc00, 0xaa00aa00, 0xff00ff00, 0x80808080, 0xc0c0c0c0, 0xa0a0a0a0, 0xf0f0f0f0,
                0x88888888, 0xcccccccc, 0xaaaaaaaa, 0xffffffff,
            ],
        ];
        // Make sure we have enough room:
        assert!(samples.len() == num_pixel_sample_samples * self.num_pixel_samples);

        let scramble = Vec2 {
            x: self.rng.uniform_u32(),
            y: self.rng.uniform_u32(),
        };
        greycode_sample_2d(&SOBOL, scramble, samples);
        // Now we should go ahead and shuffle the values:
        for i in 0..self.num_pixel_samples {
            let start_index = i * num_pixel_sample_samples;
            let end_index = start_index + num_pixel_sample_samples;
            let sub_samples = &mut samples[start_index..end_index];
            shuffle(sub_samples, 1, &mut self.rng);
        }
        shuffle(samples, num_pixel_sample_samples, &mut self.rng);
    }
}

impl Sampler for ZeroTwo {
    type ParamType = ();

    fn new(
        // Not used in our case
        _: Self::ParamType,
        num_pixel_samples: usize,
        num_dim: usize,
        arr_sizes_1d: &[usize],
        arr_sizes_2d: &[usize],
    ) -> Self {
        // Update the number of pixel samples. We generate better samples
        // when this number is a power of 2:
        let num_pixel_samples = next_pow2_u64(num_pixel_samples as u64) as usize;

        // Allocates the memory needed for the sampler (uninitialized, they will be
        // initialized when pixel start is called):

        let (samples_1d, samples_2d) = {
            let num_samples = num_pixel_samples * num_dim;
            unsafe { (mem::uninit_vec(num_samples), mem::uninit_vec(num_samples)) }
        };

        let mut arr_samples_1d = Vec::with_capacity(arr_sizes_1d.len());
        for &n in arr_sizes_1d {
            unsafe {
                arr_samples_1d.push((n, mem::uninit_vec(n * num_pixel_samples)));
            }
        }

        let mut arr_samples_2d = Vec::with_capacity(arr_sizes_2d.len());
        for &n in arr_sizes_2d {
            unsafe {
                arr_samples_2d.push((n, mem::uninit_vec(n * num_pixel_samples)));
            }
        }

        ZeroTwo {
            num_pixel_samples,
            num_dim,
            samples_1d,
            samples_2d,
            arr_samples_1d,
            arr_samples_2d,

            index_pixel_sample: 0,
            index_arr_1d: 0,
            index_arr_2d: 0,
            index_1d: 0,
            index_2d: 0,

            rng: RandGen::new_default(),
        }
    }

    fn start_pixel(&mut self, _: Vec2<usize>) {
        // Go through and generate the samples for both 1D and 2D values:

        for dim in 0..self.num_dim {
            // Loop over each dimension:
            let start_index = dim * self.num_pixel_samples;
            let end_index = start_index + self.num_pixel_samples;
            let pixel_samples = &mut self.samples_1d[start_index..end_index];
            self.gen_1d_samples(1, pixel_samples);
        }

        for dim in 0..self.num_dim {
            let start_index = dim * self.num_pixel_samples;
            let end_index = start_index + self.num_pixel_samples;
            let pixel_samples = &mut self.samples_2d[start_index..end_index];
            self.gen_2d_samples(1, pixel_samples);
        }

        for (n, arrays) in self.arr_samples_1d.iter_mut() {
            self.gen_1d_samples(*n, arrays);
        }

        for (n, arrays) in self.arr_samples_2d.iter_mut() {
            self.gen_2d_samples(*n, arrays);
        }

        // Start at the beginning again:
        self.index_pixel_sample = 0;
        self.index_arr_1d = 0;
        self.index_arr_2d = 0;
        self.index_1d = 0;
        self.index_2d = 0;
    }

    fn start_tile(&mut self, seed: u64) {
        self.rng = RandGen::new(seed);
    }

    fn get_num_pixel_samples(&self) -> usize {
        self.num_pixel_samples
    }

    fn next_pixel_sample(&mut self) -> bool {
        self.index_arr_1d = 0;
        self.index_arr_2d = 0;
        self.index_1d = 0;
        self.index_2d = 0;

        self.index_pixel_sample += 1;
        self.index_pixel_sample < self.num_pixel_samples
    }

    fn get_1d(&mut self) -> f64 {
        if self.index_1d < self.num_dim {
            let index = self.index_1d * self.num_pixel_samples + self.index_pixel_sample;
            let sample = self.samples_1d[index];
            self.index_1d += 1;
            sample
        } else {
            // If we don't have more samples in our array then we just go ahead
            // and return two uniform floats:
            self.rng.uniform_f64()
        }
    }

    fn get_2d(&mut self) -> Vec2<f64> {
        if self.index_2d < self.num_dim {
            let index = self.index_2d * self.num_pixel_samples + self.index_pixel_sample;
            let sample = self.samples_2d[index];
            self.index_2d += 1;
            sample
        } else {
            // If we don't have more samples in our array then we just go ahead
            // and return two uniform floats:
            Vec2 {
                x: self.rng.uniform_f64(),
                y: self.rng.uniform_f64(),
            }
        }
    }

    fn get_1d_array(&mut self) -> &[f64] {
        match self.arr_samples_1d.get(self.index_arr_1d) {
            Some((n, arr)) => {
                self.index_arr_1d += 1;
                let start_index = n * self.index_pixel_sample;
                let end_index = start_index + n;
                &arr[start_index..end_index]
            }
            _ => &[],
        }
    }

    fn get_2d_array(&mut self) -> &[Vec2<f64>] {
        match self.arr_samples_2d.get(self.index_arr_2d) {
            Some((n, arr)) => {
                self.index_arr_2d += 1;
                let start_index = n * self.index_pixel_sample;
                let end_index = start_index + n;
                &arr[start_index..end_index]
            }
            _ => &[],
        }
    }
}

// Fills the samples slice with samples were generated using the generator matrix gen_mat.
//
// gen_mat: the generator matrix used with generating the samples
// scramble: number used to randomly scramble the result
// samples: slice we will store the samples into
fn greycode_sample_1d(gen_mat: &[u32; 32], scramble: u32, samples: &mut [f64]) {
    let mut v = scramble;
    for (i, s) in samples.iter_mut().enumerate() {
        *s = ((v as f64) * 2.3283064365386963e-10).min(f64::ONE_MINUS_EPS);
        let index = (i + 1) as u32;
        let index = index.trailing_zeros() as usize;
        v ^= gen_mat[index];
    }
}

// Same as greycode_sample_1d but for 2D samples (see greycode_sample_1d for more details).
fn greycode_sample_2d(gen_mat: &[[u32; 32]; 2], scramble: Vec2<u32>, samples: &mut [Vec2<f64>]) {
    let mut v = [scramble.x, scramble.y];
    for (i, s) in samples.iter_mut().enumerate() {
        s.x = ((v[0] as f64) * 2.3283064365386963e-10).min(f64::ONE_MINUS_EPS);
        s.y = ((v[1] as f64) * 2.3283064365386963e-10).min(f64::ONE_MINUS_EPS);
        let index = (i + 1) as u32;
        let index = index.trailing_zeros() as usize;
        v[0] ^= gen_mat[0][index];
        v[1] ^= gen_mat[1][index];
    }
}
