// Represents color in the renderer:

use num_traits::clamp;

use pmath::vector::Vec3;
use std::ops::{Add, Div, Index, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn from_vec3(v: Vec3<f64>) -> Self {
        Color {
            r: v.x,
            g: v.y,
            b: v.z,
        }
    }

    pub fn from_scalar(s: f64) -> Self {
        Color { r: s, g: s, b: s }
    }

    // Just a fancy way of returning 0 for everything:
    pub fn black() -> Self {
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
        }
    }

    pub fn white() -> Self {
        Color {
            r: 1.,
            g: 1.,
            b: 1.,
        }
    }

    // Multiplies all of the components by the scale value:
    pub fn scale(self, s: f64) -> Self {
        Color {
            r: self.r * s,
            g: self.g * s,
            b: self.b * s,
        }
    }

    // Divides all of the components by the scale value:
    pub fn div_scale(self, s: f64) -> Self {
        Color {
            r: self.r / s,
            g: self.g / s,
            b: self.b / s,
        }
    }

    pub fn is_black(self) -> bool {
        self.r == 0. && self.g == 0. && self.b == 0.
    }

    pub fn sqrt(self) -> Self {
        Color {
            r: self.r.sqrt(),
            g: self.g.sqrt(),
            b: self.b.sqrt(),
        }
    }

    pub fn pow(self, p: f64) -> Self {
        Color {
            r: self.r.powf(p),
            g: self.g.powf(p),
            b: self.b.powf(p),
        }
    }

    pub fn exp(self) -> Self {
        Color {
            r: self.r.exp(),
            g: self.g.exp(),
            b: self.b.exp(),
        }
    }

    pub fn lerp(self, s2: Self, t: f64) -> Self {
        self.scale(1. - t) + s2.scale(t)
    }

    pub fn clamp(self, low: f64, high: f64) -> Self {
        Color {
            r: clamp(self.r, low, high),
            g: clamp(self.g, low, high),
            b: clamp(self.b, low, high),
        }
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Color {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Color {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Index<usize> for Color {
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
