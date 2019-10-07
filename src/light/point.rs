use crate::light::{LightType, Light};

pub struct Point {

}

impl Point {
     const LIGHT_TYPE: LightType = LightType::DELTA_POSITION | LightType::DELTA_POSITION;
}

impl Light for Point {

}