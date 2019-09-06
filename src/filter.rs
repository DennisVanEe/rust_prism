// This file holds a bunch of filters that can be sampled and whatnot:

use crate::math::vector::Vec2;

pub trait Filter {
    fn eval(&self, p: Vec2<f64>) -> f64;
    fn get_radius(&self) -> Vec2<f64>;
}

// Some filters that PRISM supports:

//
// Box Filter
//

#[derive(Clone, Copy)]
pub struct BoxFilter {
    radius: Vec2<f64>,
}

impl BoxFilter {
    pub fn new(radius: Vec2<f64>) -> Self {
        BoxFilter { radius }
    }
}

impl Filter for BoxFilter {
    fn eval(&self, p: Vec2<f64>) -> f64 {
        1.
    }

    fn get_radius(&self) -> Vec2<f64> {
        self.radius
    }
}

//
// Triangle Filter
//

#[derive(Clone, Copy)]
pub struct TriangleFilter {
    radius: Vec2<f64>,
}

impl TriangleFilter {
    pub fn new(radius: Vec2<f64>) -> Self {
        TriangleFilter { radius }
    }
}

impl Filter for TriangleFilter {
    fn eval(&self, p: Vec2<f64>) -> f64 {
        let e = (self.radius - p.abs()).max(Vec2::zero());
        e.x * e.y
    }

    fn get_radius(&self) -> Vec2<f64> {
        self.radius
    }
}

//
// Gaussian Filter
//

#[derive(Clone, Copy)]
pub struct GaussianFilter {
    radius: Vec2<f64>,
    exp: Vec2<f64>,
    alpha: f64,
}

impl GaussianFilter {
    pub fn new(radius: Vec2<f64>, alpha: f64) -> Self {
        GaussianFilter { 
            radius,
            exp: (radius * radius).scale(-alpha).exp(),
            alpha,
        }
    }

    fn gaussian(&self, d: f64, expv: f64) -> f64 {
        ((-self.alpha * d * d).exp() - expv).max(0.)
    }
}

impl Filter for GaussianFilter {
    fn eval(&self, p: Vec2<f64>) -> f64 {
        self.gaussian(p.x, self.exp.x) * self.gaussian(p.y, self.exp.y)
    }

    fn get_radius(&self) -> Vec2<f64> {
        self.radius
    }
}