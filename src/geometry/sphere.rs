use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::Vec3;
use crate::math::util::quadratic;
use crate::transform::Transform;

use num_traits::clamp;

use std::f64;

pub struct Sphere<T: Transform> {
    // These are pretty easy to invert, so we don't
    // have to bother storing both bits of information in this case:
    geom_to_world: T,

    radius: f64,
    z_min: f64,
    z_max: f64,
    theta_min: f64,
    theta_max: f64,
    phi_max: f64,

    // If true, the bools point outwards, if false, they point inwards:
    rev_orientation: bool,
}

impl<T: Transform> Sphere<T> {
    pub fn new(
        geom_to_world: T,
        rev_orientation: bool,
        radius: f64,
        z_min: f64,
        z_max: f64,
        phi_max: f64,
    ) -> Self {
        let z_min = clamp(z_min, -radius, radius);
        let z_max = clamp(z_max, -radius, radius);

        let theta_min = clamp(z_min / radius, -1., 1.).acos();
        let theta_max = clamp(z_max / radius, -1., 1.).acos();

        let phi_max = clamp(phi_max, 0., 360.).to_radians();

        Sphere {
            geom_to_world,
            radius,
            z_min,
            z_max,
            theta_min,
            theta_max,
            phi_max,
            rev_orientation,
        }
    }
}

impl<T: Transform> Geometry for Sphere<T> {
    fn geom_bound(&self) -> BBox3<f64> {
        BBox3::from_pnts(
            Vec3 {
                x: -self.radius,
                y: self.radius,
                z: self.z_min,
            },
            Vec3 {
                x: self.radius,
                y: self.radius,
                z: self.z_max,
            },
        )
    }

    fn world_bound(&self, t: f64) -> BBox3<f64> {
        self.geom_to_world.bbox(self.geom_bound(), t)
    }

    fn intersect(&self, ray: Ray<f64>, max_time: f64, curr_time: f64) -> Option<Interaction> {
        // Because of the way this works, we perform this operation first:
        let int_geom_to_world = self.geom_to_world.interpolate(curr_time);

        // Transform the ray to the appropriate space:
        let ray = int_geom_to_world.inverse().ray(ray);
        
        // Now we need to solve the following quadratic equation:
        let a = ray.dir.dot(ray.dir);
        let b = 2. * ray.dir.dot(ray.org);
        let c = ray.org.dot(ray.org) - self.radius * self.radius;

        let (t0, t1) = match quadratic(a, b, c) {
            Some(t) => t,
            _ => return None,
        };

        if t0 > max_time || t1 <= 0. {
            return None;
        }

        let t = if t0 <= 0. { t1 } else { t0 };

        let p_hit = ray.org + ray.dir.scale(t);
        // To generate a more robust intersection
        let p_hit = p_hit.scale(self.radius / p_hit.length());
        let p_hit = if p_hit.x == 0. && p_hit.y == 0. {
            Vec3 { x: 1e-5 * self.radius, y: p_hit.y, z: p_hit.z }
        } else {
            p_hit
        };

        let phi = p_hit.y.atan2(p_hit.x);
        let phi = if phi < 0. {
            phi + 2. * f64::consts::PI
        } else {
            phi
        };


    }
}
