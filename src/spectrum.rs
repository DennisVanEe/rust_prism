// Represents color in the renderer:

use num_traits::{clamp, Float};

use std::mem::MaybeUninit;
use std::ops::{Add, Index, Sub};

// The CIE XYZ color space:
#[derive(Clone, Copy)]
pub struct XYZColor {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl XYZColor {
    pub fn zero() -> Self {
        XYZColor {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl Add for XYZColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        XYZColor {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for XYZColor {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        XYZColor {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Index<usize> for XYZColor {
    type Output = f64;

    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of range for Vec"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct RGBColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl RGBColor {
    pub fn is_black(self) -> bool {
        self.r == 0. && self.g == 0. && self.b == 0.
    }

    pub fn sqrt(self) -> Self {
        RGBColor {
            r: self.r.sqrt(),
            g: self.g.sqrt(),
            b: self.b.sqrt(),
        }
    }

    pub fn pow(self, p: f64) -> Self {
        RGBColor {
            r: self.r.powf(p),
            g: self.g.powf(p),
            b: self.b.powf(p),
        }
    }

    pub fn exp(self) -> Self {
        RGBColor {
            r: self.r.exp(),
            g: self.g.exp(),
            b: self.b.exp(),
        }
    }

    pub fn scale(self, s: f64) -> Self {
        RGBColor {
            r: self.r * s,
            g: self.g * s,
            b: self.b * s,
        }
    }

    pub fn lerp(self, s2: Self, t: f64) -> Self {
        self.scale(1. - t) + s2.scale(t)
    }

    pub fn clamp(self, low: f64, high: f64) -> Self {
        RGBColor {
            r: clamp(self.r, low, high),
            g: clamp(self.g, low, high),
            b: clamp(self.b, low, high),
        }
    }

    // Generates CIE 1931 XYZ color space result:
    pub fn to_xyz(self) -> XYZColor {
        XYZColor {
            x: 0.412453 * self.r + 0.357580 * self.g + 0.180423 * self.b,
            y: 0.212671 * self.r + 0.715160 * self.g + 0.072169 * self.b,
            z: 0.019334 * self.r + 0.119193 * self.g + 0.950227 * self.b,
        }
    }

    // Only generates the Y value (as this is often used):
    pub fn to_y(self) -> f64 {
        0.212671 * self.r + 0.715160 * self.g + 0.072169 * self.b
    }

    pub fn from_xyz(xyz: XYZColor) -> Self {
        RGBColor {
            r: 3.240479 * xyz.x - 1.537150 * xyz.y - 0.498535 * xyz.z,
            g: -0.969256 * xyz.x + 1.875991 * xyz.y + 0.041556 * xyz.z,
            b: 0.055648 * xyz.x - 0.204043 * xyz.y + 1.057311 * xyz.z,
        }
    }
}

impl Add for RGBColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        RGBColor {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Sub for RGBColor {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        RGBColor {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl Index<usize> for RGBColor {
    type Output = f64;

    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            _ => panic!("Index out of range for Vec"),
        }
    }
}
