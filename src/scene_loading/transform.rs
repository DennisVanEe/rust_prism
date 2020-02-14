use crate::math::vector::Vec3;
use crate::math::matrix::Mat4;
use crate::transform::{AnimatedTransform, StaticTransform};

// Used to define the different animation types:
use serde::Deserialize;

use serde_json::{self, Value};
use simple_error::{bail, SimpleResult};

#[derive(Copy, Deserialize)]
struct Translation {
    translation: [f64; 3],
}

impl Translation {
    pub fn convert(self) -> SimpleResult<StaticTransform> {
        Ok(StaticTransform::new_translate(Vec3::from_arr(self.translation)))
    }
}

#[derive(Copy, Deserialize)]
struct Rotation {
    degrees: f64,
    axis: [f64; 3],
}

impl Rotation {
    pub fn convert(self) -> SimpleResult<StaticTransform> {
        Ok(StaticTransform::new_rotate(self.degrees, Vec3::from_arr(self.axis)))
    }
}

#[derive(Copy, Deserialize)]
struct Scale {
    scale: [f64; 3],
}

impl Scale {
    pub fn convert(self) -> SimpleResult<StaticTransform> {
        Ok(StaticTransform::new_scale(Vec3::from_arr(self.scale)))
    }
}

#[derive(Copy, Deserialize)]
struct Matrix {
    matrix: [f64; 16],
}

impl Matrix {
    pub fn convert(self) -> SimpleResult<StaticTransform> {
        // Make sure that the array provided is invertible:
        if let Some(m) = StaticTransform::new_matrix(Mat4::from_arr(self.matrix)) {
            Ok(m)
        } else {
            bail!("provided matrix is not invertible")
        }
    }
}

//
// These are special cases and don't have their own conversion system:

#[derive(Deserialize)]
struct Composite {
    pub composite: Vec<Value>,
}

#[derive(Deserialize)]
struct Animated {
    pub start_transform: Value,
    pub end_transform: Value,
    pub start_time: f64,
    pub end_time: f64,
}

// This is used so we can avoid dynamic allocation for now:
pub enum TransformType {
    Animated(AnimatedTransform),
    Static(StaticTransform),
}

// Given a json object that is a transform, this will parse it into one:
pub fn parse_transform(json: Value) -> SimpleResult<TransformType> {

    // We just want to parse static transformations:
    pub fn parse_static_transform(json: Value) -> SimpleResult<StaticTransform> {
        // A big if-else statement to figure out which type it is (not the most elegent solution...):

        if let Ok(t) = serde_json::from_value::<Translation>(json) {
            return t.convert();
        }
        if let Ok(t) = serde_json::from_value::<Rotation>(json) {
            return t.convert();
        }
        if let Ok(t) = serde_json::from_value::<Scale>(json) {
            return t.convert();
        }
        if let Ok(t) = serde_json::from_value::<Matrix>(json) {
            return t.convert();
        }
        if let Ok(t) = serde_json::from_value::<Composite>(json) {
            let comp = StaticTransform::new_identity();
            // Then go ahead and parse the other two:
            for j in t.composite.into_iter().rev() {
                let trans = parse_static_transform(j)?;
                comp = trans * comp;
            }
            return Ok(comp);
        }

        bail!("transform of unknown type or incorrectly formatted")
    }

    // First check if it's an animated transform:
    if let Ok(t) = serde_json::from_value::<Animated>(json) {
        let start = parse_static_transform(t.start_transform)?;
        let end = parse_static_transform(t.end_transform)?;
        if t.start_time < 0. || t.end_time < 0. || t.start_time > t.end_time {
            bail!("animated transform with invalid times")
        }
        Ok(TransformType::Animated(AnimatedTransform::new(
            start,
            end,
            t.start_time,
            t.end_time,
        )))
    } else {
        Ok(TransformType::Static(parse_static_transform(json)?))
    }
}