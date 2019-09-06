use crate::math::vector::Vec2;
use crate::spectrum::XYZColor;
use crate::filter::Filter;

// Used when performing filtered importance sampling:
struct FilterTable {
    
}

struct Pixel {
    value: XYZColor, // the 
    count: u64,
}

pub struct Film<F: Filter> {
    filter: F,
}