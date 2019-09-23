use crate::math::ray::Ray;

// This is the type that will be intersected:

pub trait AggregateObject {
    
}

pub trait Aggregate<O: AggregateObject, P> {
    fn intersect_test(&self, ray: Ray<f64>, max_time: f64, int_info: P) -> bool;
}