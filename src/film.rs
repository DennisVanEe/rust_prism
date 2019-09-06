use crate::math::vector::Vec2;
use crate::spectrum::XYZColor;
use crate::filter::Filter;

// This generates a position for where to draw the next sample from 
// for a specific pixel:

const FILTER_TABLE_WIDTH: usize = 32;
struct PixelFilter {
 
}

impl PixelFilter {
    pub fn new<T: Filter>(filter: &T) {
        let radius = filter.get_radius();
        // First we should go through and discretize the filter:
        // TODO: fix this so that we also handle the negative case:
        let mut filter_entries = [[0.; FILTER_TABLE_WIDTH]; FILTER_TABLE_WIDTH];
        for (y, row) in filter_entries.iter_mut().enumerate() {
            for (x, entry) in row.iter_mut().enumerate() {
                let x = x as f64;
                let y = y as f64;
                let p = Vec2 {
                    x: (x + 0.5) * radius.x / (FILTER_TABLE_WIDTH as f64),
                    y: (y + 0.5) * radius.y / (FILTER_TABLE_WIDTH as f64),
                };
                *entry = filter.eval(p);
            }
        }

        // Now we want to sum up all of the values in filter_entries and then
        // normalize filter entries with this:
        let filter_sum = filter_entries.iter().fold(0., |total, row| { total + row.iter().sum::<f64>() });
        filter_entries.iter_mut().for_each(|row| { row.iter_mut().for_each(|entry| { *entry = *entry / filter_sum; } ); });
    }
}

struct Pixel {
    value: XYZColor, // the 
    count: u64,
}

pub struct Film<F: Filter> {
    filter: F,
}