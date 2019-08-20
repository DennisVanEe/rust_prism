
use crate::math::transform::{AnimatedTransform, Transform};
use crate::math::vector::Vec3f;
use crate::math::matrix::Mat4f;

use serde_json::{Value, Map};
use simple_error::{bail, try_with, SimpleResult};

// Used to specify the type of transformation
// that we received:
pub enum TransformType {
    Animated(AnimatedTransform<f32>),
    Static(Transform<f32>),
}

// Top level function, returns a proper transform as expected:
pub fn parse_transform(value: Value) -> SimpleResult<TransformType> {

    // First we check the type of transform it is:


}

// Different functions for parsing different types of transformations:

pub fn parse_translate(value: Map<String, Value>) -> SimpleResult<Transform<f32>> {
    // Check to make sure it has the correct parameter specified:
    let vector = match value.get("vec") {
        Some(vec) => vec,
        _ => bail!("translate transform missing vec property"),
    };

    let vector = match vector {
        Value::Array(vec) => vec,
        _ => bail!("ill-formed translate transform vec property"),
    };

    // TODO: finish this part here...
}