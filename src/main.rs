// Clean this stuff up in the future...
// This is here just for now.

mod scene_loading;
mod bvh;
mod camera;
mod film;
mod filter;
mod geometry;
mod integrator;
mod light;
mod math;
mod memory;
mod pixel_buffer;
mod sampler;
mod scene;
mod shading;
mod spectrum;
mod transform;

use std::env;

fn main() {

}

static HELP_MSG: &str = 
"
    -scene:\tThe TOML file with the scene description.\n
";

// Parse any arguments we may have:
fn parse_commands() {
    
    
    todo!();
}