use crate::math::numbers::Float;
use crate::math::random::RandGen;
use crate::math::vector::Vec2;
use crate::memory::uninit_vec;
use crate::sampler::{shuffle, Sampler};

pub struct ZeroTwo {
    n_pxl_sampl: usize,
    n_dim: usize,

    // The samples values are stored contiguously in memory but represents
    // a 2D array of values.
    // They are represented as such (first index which dimension, then which pixel sample we care about):
    //
    // dim0: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // dim1: pixel_sample0, pixel_sample1, pixel_sample2, ...
    // ...
    sampl_1d: Vec<f32>,
    sampl_2d: Vec<Vec2<f32>>,

    arr_sampl_1d: Vec<Vec<f32>>,
    arr_sampl_2d: Vec<Vec<Vec2<f32>>>,

    rng: RandGen,
}

impl ZeroTwo {
    // n_pxl_sampl: number of pixel samples (samples per pixel)
    // n_dim: number of dimensions we are planning on sampling:
    pub fn new(
        n_pxl_sampl: usize,
        n_dim: usize,
        arr_sampl_1d: Vec<Vec<f32>>,
        arr_sampl_2d: Vec<Vec<Vec2<f32>>>,
    ) -> Self {
        // Update the number of pixel samples. We generate better samples
        // when this number is a power of 2:
        let n_pxl_sampl = roundup_pow2(n_pxl_sampl);

        // Allocate buffers for the "regular" samples_1d and samples_2d values:
        let (sampl_1d, sampl_2d) = {
            let n_sampl = n_pxl_sampl * n_dim;
            unsafe { (uninit_vec(n_sampl), uninit_vec(n_sampl)) }
        };

        ZeroTwo {
            n_pxl_sampl,
            n_dim,
            sampl_1d,
            sampl_2d,
            arr_sampl_1d,
            arr_sampl_2d,
            rng: RandGen::new(),
        }
    }

    fn gen_1d_samples(&mut self, num_pixel_sample_samples: usize, samples: &mut [f32]) {
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

    fn gen_2d_samples(&mut self, num_pixel_sample_samples: usize, samples: &mut [Vec2<f32>]) {
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
    }
}

// Fills the samples slice with samples were generated using the generator matrix gen_mat.
//
// gen_mat: the generator matrix used with generating the samples
// scramble: number used to randomly scramble the result
// samples: slice we will store the samples into
fn greycode_sample_1d(gen_mat: &[u32; 32], scramble: u32, samples: &mut [f32]) {
    let mut v = scramble;
    for (i, s) in samples.iter_mut().enumerate() {
        *s = ((v as f32) * 2.3283064365386963e-10).min(f32::ONE_MINUS_EPS);
        let index = (i + 1) as u32;
        let index = index.trailing_zeros() as usize;
        v ^= unsafe { *gen_mat.get_unchecked(index) };
    }
}

// Same as greycode_sample_1d but for 2D samples (see greycode_sample_1d for more details).
fn greycode_sample_2d(gen_mat: &[[u32; 32]; 2], scramble: Vec2<u32>, samples: &mut [Vec2<f32>]) {
    let mut v = [scramble.x, scramble.y];
    for (i, s) in samples.iter_mut().enumerate() {
        s.x = ((v[0] as f32) * 2.3283064365386963e-10).min(f32::ONE_MINUS_EPS);
        s.y = ((v[1] as f32) * 2.3283064365386963e-10).min(f32::ONE_MINUS_EPS);
        let index = (i + 1) as u32;
        let index = index.trailing_zeros() as usize;
        v[0] ^= unsafe { *gen_mat[0].get_unchecked(index) };
        v[1] ^= unsafe { *gen_mat[1].get_unchecked(index) };
    }
}

fn roundup_pow2(n: usize) -> usize {
    let n = n as u64;
    let n = n - 1;
    let n = n | n >> 1;
    let n = n | n >> 2;
    let n = n | n >> 4;
    let n = n | n >> 8;
    let n = n | n >> 16;
    let n = n | n >> 32;
    (n + 1) as usize
}
