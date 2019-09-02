// A spectrum object is a special object that stores
// information regarding the spectrum.

use num_traits::{clamp, Float};

use std::mem::MaybeUninit;
use std::ops::{Add, Sub, Index};

const NUM_SPECTRUM_SAMPLES: usize = 24;

#[derive(Clone, Copy)]
struct Spectrum<T: Float> {
    data: [T; NUM_SPECTRUM_SAMPLES],
}

impl<T: Float> Spectrum<T> {
    pub fn is_black(self) -> bool {
        for &x in &self.data {
            if x != T::zero() {
                return false;
            }
        }
        true
    }

    pub fn sqrt(self) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for (result, d) in data.iter_mut().zip(&self.data) {
            *result = d.sqrt();
        }
        Spectrum { data }
    }

    pub fn pow(self, p: T) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for (result, d) in data.iter_mut().zip(&self.data) {
            *result = d.powf(p);
        }
        Spectrum { data }
    }

    pub fn exp(self) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for (result, d) in data.iter_mut().zip(&self.data) {
            *result = d.exp();
        }
        Spectrum { data }
    }

    pub fn scale(self, s: T) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for (result, &d) in data.iter_mut().zip(&self.data) {
            *result = d * s;
        }
        Spectrum { data }
    }

    pub fn lerp(self, s2: Self, t: T) -> Self {
        self.scale(T::one() - t) + s2.scale(t)
    }

    pub fn clamp(self, low: T, high: T) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for (result, &d) in data.iter_mut().zip(&self.data) {
            *result = clamp(d, low, high);
        }
        Spectrum { data }
    }
}

impl<T: Float> Add for Spectrum<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for ((result, &lhs), &rhs) in data.iter_mut().zip(&self.data).zip(&rhs.data) {
            *result = lhs + rhs;
        }
        Spectrum { data }
    }
}

impl<T: Float> Sub for Spectrum<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut data: [T; NUM_SPECTRUM_SAMPLES] = unsafe { MaybeUninit::uninit().assume_init() };
        for ((result, &lhs), &rhs) in data.iter_mut().zip(&self.data).zip(&rhs.data) {
            *result = lhs - rhs;
        }
        Spectrum { data }
    }
}

impl<T: Float> Index<usize> for Spectrum<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.data[i]
    }
}