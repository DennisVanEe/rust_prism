
use crate::math::transform::{AnimatedTransform, Transform};
use crate::math::vector::{Vec3, Vec4};
use crate::math::matrix::Mat4;

use std::mem::MaybeUninit;

use serde_json::{Value, Map, Number};
use simple_error::{bail, try_with, SimpleResult};

// Used to specify the type of transformation
// that we received:
pub enum TransformType {
    Animated(AnimatedTransform<f64>),
    Static(Transform<f64>),
}

// Given a json object that is a transform, this will parse it into one:
pub fn parse_transform(json_transf: &Map<String, Value>) -> SimpleResult<TransformType> {
    pub fn parse_non_animated_transform(json_transf: &Map<String, Value>) -> SimpleResult<Transform<f64>> {
        // First we check what the top level transform is:
        let json_transf_type = match json_transf.get("type") {
            Some(t) => t,
            _ => bail!("transform missing type property"),
        };
        let transf_type = match json_transf_type {
            Value::String(t) => t,
            _ => bail!("transform type must be a string"),
        };

        // Handle the case where it is a composite transform:
        if transf_type == "composite" {
            let mut composite_transf = Transform::new_identity();
            // Extract the array:
            let json_transf_array = match json_transf.get("transf") {
                Some(t) => t,
                _ => bail!("composite transform missing transf property"),
            };
            let json_transf_array = match json_transf_array {
                Value::Array(t) => t,
                _ => bail!("ill-formed composite transform transf property"),
            };
            // We iterate over it backwards so that when we multiply the values to combine
            // the transformations, the correct application order is achieved:
            for j_transf in json_transf_array.iter() {
                let j_transf = match j_transf {
                    Value::Object(t) => t,
                    _ =>bail!("ill-formed composite transform transf property"),
                };
                let transf = match parse_non_animated_transform(j_transf) {
                    Ok(t) => t,
                    error => return error,
                };
                composite_transf = transf * composite_transf;
            }
            return Ok(composite_transf);
        }

        // Match it with normal transformations now:
        match transf_type.as_ref() {
            "translate" => parse_translate(json_transf),
            "rotate" => parse_rotate(json_transf),
            "scale" => parse_scale(json_transf),
            "animated" => bail!("animated transforms can only appear as a top-level transformation"),
            _ => bail!("unknown transform type detected"),
        }
    }

    // Check if the top-level transform is animated:
    let json_transf_type = match json_transf.get("type") {
        Some(t) => t,
        _ => bail!("transform missing type property"),
    };
    let transf_type = match json_transf_type {
        Value::String(t) => t,
        _ => bail!("transform type must be a string"),
    };

    if transf_type == "animated" {
        // Get the necessary components:
        let json_start_transf = match json_transf.get("start_transf") {
            Some(s) => s,
            _ => bail!("animated transform missing start_transf property"),
        };
        let json_end_transf = match json_transf.get("end_transf") {
            Some(e) => e,
            _ => bail!("animated transform missing end_transf property"),
        };
        let json_start_time = match json_transf.get("start_time") {
            Some(s) => s,
            _ => bail!("animated transform missing start_time property"),
        };
        let json_end_time = match json_transf.get("end_time") {
            Some(e) => e,
            _ => bail!("animated transform missing end_time property"),
        };

        // Now make sure we convert them to the correct type:
        let json_start_transf = match json_start_transf {
            Value::Object(s) => s,
            _ => bail!("ill-formed animated transform start_transf property"),
        };
        let json_end_transf = match json_end_transf {
            Value::Object(e) => e,
            _ => bail!("ill-formed animated transform end_transf property"),
        };

        // Now we further process these values:
        let start_transf = match parse_non_animated_transform(json_start_transf) {
            Ok(s) => s,
            Err(m) => bail!("{}", m),
        };
        let end_transf = match parse_non_animated_transform(json_end_transf) {
            Ok(e) => e,
            Err(m) => bail!("{}", m),
        };
        let start_time = match parse_number(json_start_time) {
            Some(s) => s,
            _ => bail!("ill-formed animated transform start_time property"),
        };
        let end_time = match parse_number(json_end_time) {
            Some(e) => e,
            _ => bail!("ill-formed animated transform end_time property"),
        };

        match AnimatedTransform::new(start_transf, end_transf, start_time, end_time) {
            Some(r) => Ok(TransformType::Animated(r)),
            _ => bail!("non-invertible matrix detected when decomposing transformation"),
        }
    } else {
        match parse_non_animated_transform(json_transf) {
            Ok(t) => Ok(TransformType::Static(t)),
            Err(m) => bail!("{}", m),
        }
    }
}

// A function for extracting float values from a value:
fn parse_number(json_number: &Value) -> Option<f64> {
    match json_number {
        Value::Number(n) => n.as_f64(),
        _ => None,
    }
}

fn parse_vec3(json_vec: &Vec<Value>) -> Option<Vec3<f64>> {
    if json_vec.len() != 3 {
        return None;
    }

    unsafe { 
        Some(Vec3 {
            x: match parse_number(json_vec.get_unchecked(0)) {
                Some(n) => n,
                _ => return None,
            },
            y: match parse_number(json_vec.get_unchecked(1)) {
                Some(n) => n,
                _ => return None,
            },
            z: match parse_number(json_vec.get_unchecked(2)) {
                Some(n) => n,
                _ => return None,
            },
        }) 
    }
}

fn parse_array16(json_vec: &Vec<Value>) -> Option<[f64; 16]> {
    if json_vec.len() != 16 {
        return None;
    }

    let mut result: [f64; 16] = MaybeUninit::uninit().assume_init();
    for (i, json_num) in json_vec.iter().enumerate() {
        *unsafe { result.get_unchecked_mut(i) } = match parse_number(json_num) {
            Some(v) => v,
            _ => return None,
        };
    }
    Some(result)
}

fn parse_matrix(json_transf: &Map<String, Value>) -> SimpleResult<Transform<f64>> {
    let json_matrix = match json_transf.get("mat") {
        Some(m) => m,
        _ => bail!("matrix transform missing mat property"),
    };

    let json_matrix = match json_matrix {
        Value::Array(m) => m,
        _ => bail!("ill-formed matrix transform mat property"),
    };

    let matrix = match parse_array16(&json_matrix) {
        Some(m) => m,
        _ => bail!("ill-formed matrix transform mat property"),
    };

    let r0 = Vec4 {
        x: matrix[0],
        y: matrix[1],
        z: matrix[2],
        w: matrix[3],
    };
    let r1 = Vec4 {
        x: matrix[4],
        y: matrix[5],
        z: matrix[6],
        w: matrix[7],
    };
    let r2 = Vec4 {
        x: matrix[8],
        y: matrix[9],
        z: matrix[10],
        w: matrix[11],
    };
    let r3 = Vec4 {
        x: matrix[12],
        y: matrix[13],
        z: matrix[14],
        w: matrix[15],
    };
    let matrix = Mat4::new([r0, r1, r2, r3]);
    match Transform::new(matrix) {
        Some(m) => Ok(m),
        _ => bail!("matrix transform is not invertible"),
    }
}

// Different functions for parsing different types of transformations:

fn parse_translate(json_transf: &Map<String, Value>) -> SimpleResult<Transform<f64>> {
    // Check to make sure it has the correct parameter specified:
    let json_trans = match json_transf.get("trans") {
        Some(t) => t,
        _ => bail!("translate transform missing trans property"),
    };

    let json_trans = match json_trans {
        Value::Array(t) => t,
        _ => bail!("ill-formed translate transform trans property"),
    };

    let trans = match parse_vec3(json_trans) {
        Some(t) => t,
        _ => bail!("ill-formed translate transform trans property"),
    };

    Ok(Transform::new_translate(trans))
}

fn parse_scale(json_transf: &Map<String, Value>) -> SimpleResult<Transform<f64>> {
    let json_scale = match json_transf.get("scale") {
        Some(s) => s,
         _ => bail!("scale transform missing scale property"),
    };

    let json_scale = match json_scale {
        Value::Array(s) => s,
        _ => bail!("ill-formed scale transform scale property"),
    };

    let scale = match parse_vec3(json_scale) {
        Some(s) => s,
        _ => bail!("ill-formed scale transform scale property"),
    };

    Ok(Transform::new_scale(scale))
}

fn parse_rotate(json_transf: &Map<String, Value>) -> SimpleResult<Transform<f64>> {
    // Check to make sure we have the correct values (and types):
    let json_degrees = match json_transf.get("degrees") {
        Some(d) => d,
        _ => bail!("rotate transform missing degrees property"),
    };

    let degrees = match parse_number(json_degrees) {
        Some(d) => d,
         _ => bail!("rotate transform missing degrees property"),
    };

    let json_axis = match json_transf.get("axis") {
        Some(a) => a,
        _ => bail!("rotate transform missing axis property"),
    };

    let json_axis = match json_axis {
        Value::Array(a) => a,
        _ => bail!("ill-formed rotate transform vec property"),
    };

    let axis = match parse_vec3(json_axis) {
        Some(a) => a,
        _ => bail!("ill-formed rotate transform vec property"),
    };

    Ok(Transform::new_rotate(degrees, axis))
}