use crate::spectrum::{RGBSpectrum, XYZColor};

use super::{Pixel, PixelType};

#[derive(Clone, Copy, Debug)]
pub struct BeautyPixel {
    pub value: XYZColor,
    pub count: u32,
}

impl Pixel for BeautyPixel {
    type FinalOutput = RGBSpectrum;
    const TypeID: PixelType = PixelType::Beauty;

    fn zero() -> Self {
        BeautyPixel {
            value: XYZColor::zero(),
            count: 0,
        }
    }

    fn set_zero(&mut self) {
        self.value = XYZColor::zero();
        self.count = 0;
    }

    fn add(&mut self, p: &Self) {
        // Relatively simple update function:
        self.value = self.value + p.value;
        self.count = self.count + p.count;
    }

    fn finalize(&self) -> Self::FinalOutput {
        // First we normalize the XYZColor value:
        let weight = 1. / (self.count as f64);
        let final_xyz = self.value.scale(weight);
        // Convert it to RGBColor space:
        RGBSpectrum::from_xyz(final_xyz)
    }
}
