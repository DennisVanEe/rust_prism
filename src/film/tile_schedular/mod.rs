use crate::math::vector::Vec2;

use super::PixelIndex;

pub mod linear;

// A TileSchedular is a program used to schedule the order of tiles to render
// by multiple threads.
pub trait TileSchedular {
    // Any custom parameters that the TileSchedular might need:
    type Param;

    // Creates a new TileSchedular:
    fn new(tile_res: Vec2<usize>, param: Self::Param) -> Self;

    // Resets the TileSchedular to its originals state. This
    // function does not have to be thread safe:
    fn reset(&mut self);

    // Retrieve the initial tile index. This function does not
    // have to be thread safe. By the off chance that there are more
    // threads than tiles, it'll return None:
    fn init_index(&mut self) -> Option<PixelIndex>;

    // Retrieves the next pixel for rendering. Returns None if no
    // such tile exists anymore. This function must be thread safe:
    fn next_index(&self, cur_index: PixelIndex) -> Option<PixelIndex>;

    // Returns the percentage of tiles that are finished at this point.
    // Doesn't have to be very accurate but does have to be thread safe:
    fn get_percent_done(&self) -> f64;
}
