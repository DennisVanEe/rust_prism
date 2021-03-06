use crate::film::{ImageBuffer, ImagePixel};
use lodepng::{self, ColorType};
use simple_error::{bail, SimpleResult};
use std::fs::File;
use std::io::prelude::*;

/// How many bits to use to encode each channel of the image.
#[derive(Clone, Copy, Debug)]
pub enum BitDepth {
    EIGHT,
    SIXTEEN,
}

/// Given an image buffer, converts it to a png file and writes it to the specified path.
pub fn write_png(image: &ImageBuffer, path: &str, bit_depth: BitDepth) -> SimpleResult<()> {
    let png_buffer = match bit_depth {
        BitDepth::EIGHT => {
            let mut buffer = Vec::with_capacity(image.buffer.len());
            for &image_pixel in image.buffer.iter() {
                buffer.push(from_image_pixel_eight(image_pixel));
            }
            match lodepng::encode_memory(&buffer, image.res.x, image.res.y, ColorType::RGB, 8) {
                Ok(result) => result,
                Err(err) => bail!("Error creating png file: {}", err),
            }
        }
        BitDepth::SIXTEEN => {
            let mut buffer = Vec::with_capacity(image.buffer.len());
            for &image_pixel in image.buffer.iter() {
                buffer.push(from_image_pixel_sixteen(image_pixel));
            }
            match lodepng::encode_memory(&buffer, image.res.x, image.res.y, ColorType::RGB, 16) {
                Ok(result) => result,
                Err(err) => bail!("Error creating png data: {}", err),
            }
        }
    };

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(err) => bail!("Error creating png file: {}", err),
    };

    if let Err(err) = file.write_all(&png_buffer) {
        bail!("Error writing png file: {}", err.to_string());
    }

    Ok(())
}

fn from_image_pixel_eight(pixel: ImagePixel) -> [u8; 3] {
    [
        f64_to_bitdepth(pixel.r, 8) as u8,
        f64_to_bitdepth(pixel.g, 8) as u8,
        f64_to_bitdepth(pixel.b, 8) as u8,
    ]
}

fn from_image_pixel_sixteen(pixel: ImagePixel) -> [u16; 3] {
    [
        f64_to_bitdepth(pixel.r, 16) as u16,
        f64_to_bitdepth(pixel.g, 16) as u16,
        f64_to_bitdepth(pixel.b, 16) as u16,
    ]
}

/// Converts a float v in range [0, 1] to a specified bith depth.
fn f64_to_bitdepth(v: f64, depth: u32) -> u32 {
    let val = if v >= 1.0 {
        (2u32.pow(depth) - 1) as f64
    } else {
        v * (2u32.pow(depth) as f64)
    }
    .floor();
    val as u32
}
