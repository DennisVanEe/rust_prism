// A Transform is something that can take a mathematical primitive and a time,
// and transform it. Such transformations must support the following trait:

pub mod static_transform;

use crate::math::vector::Vec3;
use crate::math::ray::Ray;
use crate::math::bbox::BBox3;

pub trait Transform {
    // Be able to invert itself.
    fn inverse(&self) -> Self;

    // The different values that we should be able to transform:

    fn point(&self, p: Vec3<f64>, t: f64) -> Vec3<f64>;
    fn normal(&self, n: Vec3<f64>, t: f64) -> Vec3<f64>;
    fn vector(&self, v: Vec3<f64>, t: f64) -> Vec3<f64>;
    fn ray(&self, r: Ray<f64>, t: f64) -> Ray<f64>;
    fn bbox(&self, b: BBox3<f64>, t: f64) -> Vec3<f64>;
}