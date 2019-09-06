use crate::math::vector::Vec2;
use crate::spectrum::XYZColor;
use crate::filter::Filter;

// This isn't a filter in the sense that it returns weights for the samples. Instead,
// given two uniform random variables, it will return a position based on the filter
// function passed to it. All the samples have a weight of one.

const FILTER_TABLE_WIDTH: usize = 32;
struct PixelFilter {
    // A CDF Py(x) that allows us to samle the x value:
    cdf_x: [f64; FILTER_TABLE_WIDTH],
    // A CDF P(v|u) that allows us to sample the y value:
    cdf_y: [[f64; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH],
    // Radius of the filter:
    radius: Vec2<f64>,
}

impl PixelFilter {
    pub fn new<T: Filter>(filter: &T) -> Self {
        // Filed in as follows:
        // y0: [x0, x1, x2, x3],
        // y1: [x0, x1, x2, x3],
        // y2: [x0, x1, x2, x3],
        // y3: [x0, x1, x2, x3],
        // So, to index into filter_entries, use [y][x] where y selects the row and x the entry in the row

        let radius = filter.get_radius();

        let pdf_xy = {
            // First we should go through and discretize the filter:
            let mut filter_entries = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
            for (y, row) in filter_entries.iter_mut().enumerate() {
                for (x, entry) in row.iter_mut().enumerate() {
                    let p = Vec2 {
                        x: ((x as f64 + 0.5) / FILTER_TABLE_WIDTH as f64) * (2. * radius.x) - radius.x,
                        y: ((y as f64 + 0.5) / FILTER_TABLE_WIDTH as f64) * (2. * radius.y) - radius.y,
                    };
                    *entry = filter.eval(p);
                }
            }

            // Now we want to normalize the entires by summing all of the table entries up and dividing each
            // entry by this sum. So, that we have a pdf for a specific x, y value:
            let filter_sum = filter_entries.iter().fold(0., |total, row| { total + row.iter().sum::<f64>() });
            filter_entries.iter_mut().for_each(|row| { row.iter_mut().for_each(|entry| { *entry = *entry / filter_sum; } ); });
            filter_entries
        };

        // Now we want to calculate a marginal pdf for GETTING the x values (it's p_y(x))
        let mut pdf_x = [0.; FILTER_TABLE_WIDTH];
        for (i, y_row) in pdf_xy.iter().enumerate() {
            pdf_x[i] = y_row.iter().sum();
        }
        // To sample the pdf_x distribution, we need to form a cdf (it's P_y(x)):
        let mut cdf_x = [0.; FILTER_TABLE_WIDTH];
        for (i, &prob) in pdf_x.iter().enumerate() {
            cdf_x[i..].iter_mut().for_each(|x| { *x += prob; });
        }
        // To sample the pdf_y value, we need to generate a table that, if given an x 
        // value from pdf_x, we get a y value from pdf_y (so we index into the table 
        // with the x value):
        let mut pdf_y = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for (x, x_row) in pdf_y.iter_mut().enumerate() {
            for (y, val) in x_row.iter_mut().enumerate() {
                *val = pdf_xy[y][x] / pdf_x[x];
            }
        }
        // Now we want to turn this pdf into a cdf so we can sample it:
        let mut cdf_y = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for x_row in cdf_y.iter_mut() {
            for (i, prob) in x_row.iter().enumerate() {
                x_row[i..].iter_mut().for_each(|x| { *x += prob; });
            }
        }

        PixelFilter { cdf_x, cdf_y, radius }
    }

    pub fn sample_pos(self, r1: f64, r2: f64) -> Vec2<f64> {
        // First, we sample the x-value:
        let x = self.cdf_x.iter().position(|&x| { x >= r1 }).unwrap();
        // Using this x-value, we can now find the y-value:
        let y = self.cdf_y[x].iter().position(|&y| { y >= r2 }).unwrap();

        // Convert these into points on the film plane:
        Vec2 {
            x: ((x as f64 + 0.5) / FILTER_TABLE_WIDTH as f64) * (2. * self.radius.x) - self.radius.x,
            y: ((y as f64 + 0.5) / FILTER_TABLE_WIDTH as f64) * (2. * self.radius.y) - self.radius.y,
        }
    }
}

struct Pixel {
    value: XYZColor, // the 
    count: u64,
}

pub struct Film<F: Filter> {
    filter: F,
}