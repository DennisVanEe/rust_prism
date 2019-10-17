use crate::filter::Filter;
use crate::math::vector::Vec2;
use crate::pixel_buffer::{Pixel, PixelBuffer, PixelTile, TILE_DIM};
use crate::spectrum::XYZColor;

use simple_error::{bail, SimpleResult};

use std::sync::atomic::{Ordering, AtomicUsize};
use std::mem::transmute;

// The pixel filter uses the technique described here:
// "Filter Importance Sampling" - Manfred Ernst, Marc Stamminger, Gunther Greiner
// This isn't exposed to the user, as the user just passes a filter.

const FILTER_TABLE_WIDTH: usize = 64;

#[derive(Clone, Copy)]
struct PixelFilter {
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

    pub fn sample_pos(self, r1: f64, r2: f64) -> Vec2<f64> {
        // First, we sample the x-value:
        let x = self.cdf_x.iter().position(|&cdf| cdf > r1).unwrap();
        // Using this x-value, we can now find the y-value:
        let y = self.cdf_y[x].iter().position(|&cdf| cdf >= r2).unwrap();

        // Convert these indices to x and y coordinates:
        let x = x as f64;
        let y = y as f64;
        Vec2 {
            x: (x + 0.5) / (FILTER_TABLE_WIDTH as f64) * (2. * self.radius.x) - self.radius.x,
            y: (y + 0.5) / (FILTER_TABLE_WIDTH as f64) * (2. * self.radius.y) - self.radius.y,
        }
    }
}

// Because we are using the technique above, each pixel will have a weight
// of just one. Also, each thread works exclusively on a single pixel, so
// we don't have to worry about locking it or anything:

#[derive(Clone, Copy)]
struct FilmPixel {
    pub value: XYZColor,
    pub count: u64,
}

impl Pixel for FilmPixel {
    fn zero() -> Self {
        FilmPixel {
            value: XYZColor::zero(),
            count: 0,
        }
    }

    fn update(&mut self, p: &Self) {
        self.value = self.value + p.value;
        self.count = self.count + p.count;
    }

    fn set_zero(&mut self) {
        self.value = XYZColor::zero();
        self.count = 0;
    }
}

// The film class is in charge of managing all information about the film we may want.
pub struct Film {
    pixel_buffer: PixelBuffer<FilmPixel>,
    pixel_filter: PixelFilter,

    // Specifies which task is next for a thread to work on:
    next_tile: AtomicUsize,
}

impl Film {
    // This performs the check to make sure that the resolution
    // provided is a multiple of the TILE_DIM. I could remove this
    // constraint, but that would make the code a little bit more
    // complex, and I don't feel like doing that:
    pub fn new<T: Filter>(filter: &T, pixel_res: Vec2<usize>) -> SimpleResult<Self> {
        // First we check if the resolution is a multiple of
        // the TILE_DIM:
        if pixel_res.x % TILE_DIM != 0 || pixel_res.y % TILE_DIM != 0 {
            bail!(
                "The provided Film resolution must be a multiple of: {}",
                TILE_DIM
            );
        }
        let tile_res = Vec2 {
            x: pixel_res.x / TILE_DIM,
            y: pixel_res.y / TILE_DIM,
        };
        Ok(Film {
            pixel_buffer: PixelBuffer::new_zero(tile_res),
            pixel_filter: PixelFilter::new(filter),
            next_tile: AtomicUsize::new(0),
        })
    }

    // Because this is potentially called by multiple threads, we make it an immutable borrow:
    pub fn next_tile(&self) -> Option<PixelTile<FilmPixel>> {
        // We are doing a simple scan-line approach here, so we just increment the counter. We
        // are also currently using simple uniform sampling (no adaptive sampling). With adaptive
        // sampling this could potentially be a lot more difficult. Future me will worry about that.
        //
        // Now, because of wrapping behaviour, if this gets called MANY times after we are done,
        // it could potentially wrap around and falsly return true.
        let tile_index = self.next_tile.fetch_add(1, Ordering::Relaxed);
        // Check if this is a valid tile or not:
        if tile_index >= self.pixel_buffer.get_num_tiles() {
            return None;
        }

        // Otherwise we can just create the tile:
        Some(self.pixel_buffer.get_zero_tile(tile_index))
    }

    // Updates the film buffer at the specified location with the given tiles:
    pub fn update_tile(&self, tile: &PixelTile<FilmPixel>) {
        // We are going to do something rather "tricky" here.
        // Because we know that every tile that is sent here is updating a unique
        // part of the image, we don't have to lock this function. For this reason we can do the 
        // dangerous thing we are about to do.
        //
        // TODO: figure out a cleaner way of doing this:
        let mut_self = unsafe {
            transmute::<&Self, &mut Self>(self)
        };
        // Now we can just go ahead and update the tile:
        mut_self.pixel_buffer.update_tile(tile);
    }

    pub fn get_pixel_res(&self) -> Vec2<usize> {
        self.pixel_buffer.get_pixel_res()
    }

    pub fn get_tile_res(&self) -> Vec2<usize> {
        self.pixel_buffer.get_tile_res()
    }
}