// This file stores all of the different filters that PRISM
// supports.

use pmath::vector::Vec2;
use std::hint;

pub trait Filter {
    fn eval(&self, p: Vec2<f64>) -> f64;
    fn get_radius(&self) -> Vec2<f64>;
}

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
    fn eval(&self, _: Vec2<f64>) -> f64 {
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

const FILTER_TABLE_WIDTH: usize = 64;

/// The pixel filter uses the technique described here:
/// "Filter Importance Sampling" - Manfred Ernst, Marc Stamminger, Gunther Greiner
/// Essentially we use a filter distribution to sample points on a pixel.
#[derive(Clone, Copy)]
pub struct PixelFilter {
    // A CDF Py(x) that allows us to sample the x value:
    cdf_x: [f64; FILTER_TABLE_WIDTH],
    // A CDF P(v|u) that allows us to sample the y value:
    cdf_y: [[f64; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH],
    // Radius of the filter:
    radius: Vec2<f64>,
}

impl PixelFilter {
    pub fn new<T: Filter>(filter: &T) -> Self {
        // Filed in as follows:
        // x0: [y0, y1, y2, y3],
        // x1: [y0, y1, y2, y3],
        // x2: [y0, y1, y2, y3],
        // x3: [y0, y1, y2, y3],
        // So, to index into pdf_xy, use [x][y] where y selects the row and x the entry in the row

        let radius = filter.get_radius();

        let pdf_xy = {
            // First we should go through and discretize the filter:
            let mut filter_entries = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
            for (x, row) in filter_entries.iter_mut().enumerate() {
                for (y, entry) in row.iter_mut().enumerate() {
                    let x = x as f64;
                    let y = y as f64;
                    let p = Vec2 {
                        x: (x + 0.5) / (FILTER_TABLE_WIDTH as f64) * (2. * radius.x) - radius.x,
                        y: (y + 0.5) / (FILTER_TABLE_WIDTH as f64) * (2. * radius.y) - radius.y,
                    };
                    *entry = filter.eval(p).abs();
                }
            }

            // Now we want to normalize the entires by summing all of the table entries up and dividing each
            // entry by this sum. So, that we have a pdf for a specific x, y value:
            let filter_sum = filter_entries
                .iter()
                .fold(0., |total, row| total + row.iter().sum::<f64>());
            filter_entries.iter_mut().for_each(|row| {
                row.iter_mut().for_each(|entry| {
                    *entry /= filter_sum;
                });
            });
            filter_entries
        };

        // Now we want to calculate a marginal pdf for GETTING the x values (it's p_y(x))
        let mut pdf_x = [0.; FILTER_TABLE_WIDTH];
        for (x, x_row) in pdf_xy.iter().enumerate() {
            pdf_x[x] = x_row.iter().sum();
        }
        // To sample the pdf_x distribution, we need to form a cdf (it's P_y(x)):
        let mut cdf_x = [0.; FILTER_TABLE_WIDTH];
        for (x, &pdf) in pdf_x.iter().enumerate() {
            cdf_x[x..].iter_mut().for_each(|t| {
                *t += pdf;
            });
        }
        // To sample the pdf_y value, we need to generate a table that, if given an x
        // value from pdf_x, we get a y value from pdf_y (so we index into the table
        // with the x value):
        let mut pdf_y = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for (x, x_row) in pdf_y.iter_mut().enumerate() {
            for (y, val) in x_row.iter_mut().enumerate() {
                *val = pdf_xy[x][y] / pdf_x[x];
            }
        }
        // Now we want to turn this pdf into a cdf so we can sample it:
        let mut cdf_y = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for (cdf_y_row, pdf_y_row) in cdf_y.iter_mut().zip(pdf_y.iter()) {
            for (y, &prob) in pdf_y_row.iter().enumerate() {
                cdf_y_row[y..].iter_mut().for_each(|t| {
                    *t += prob;
                });
            }
        }

        PixelFilter {
            cdf_x,
            cdf_y,
            radius,
        }
    }

    pub fn sample_pos(self, r: Vec2<f64>) -> Vec2<f64> {
        // First, we sample the x-value:
        let x = match self.cdf_x.iter().position(|&cdf| cdf > r.x) {
            Some(x) => x,
            _ => FILTER_TABLE_WIDTH / 2,
        };

        // Using this x-value, we can now find the y-value:
        let y = match self.cdf_y[x].iter().position(|&cdf| cdf >= r.y) {
            Some(y) => y,
            _ => FILTER_TABLE_WIDTH / 2,
        };

        // Convert these indices to x and y coordinates:
        let x = x as f64;
        let y = y as f64;
        Vec2 {
            x: (x + 0.5) / (FILTER_TABLE_WIDTH as f64) * (2. * self.radius.x) - self.radius.x,
            y: (y + 0.5) / (FILTER_TABLE_WIDTH as f64) * (2. * self.radius.y) - self.radius.y,
        }
    }
}
