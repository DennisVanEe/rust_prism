use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::util::quadratic;
use crate::math::vector::{Vec2, Vec3};

use num_traits::clamp;

use std::f64;

pub struct Sphere {
    radius: f64,
    z_min: f64,
    z_max: f64,
    theta_min: f64,
    theta_max: f64,
    phi_max: f64,

    // If true, the bools point outwards, if false, they point inwards:
    rev_orientation: bool,
}

impl Sphere {
    pub fn new(rev_orientation: bool, radius: f64, z_min: f64, z_max: f64, phi_max: f64) -> Self {
        let z_min = clamp(z_min, -radius, radius);
        let z_max = clamp(z_max, -radius, radius);

        let theta_min = clamp(z_min / radius, -1., 1.).acos();
        let theta_max = clamp(z_max / radius, -1., 1.).acos();

        let phi_max = clamp(phi_max, 0., 360.).to_radians();

        Sphere {
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

impl Geometry for Sphere {
    fn get_bound(&self) -> BBox3<f64> {
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

    // TODO: figure out centroid stuff:
    fn get_centroid(&self) -> Vec3<f64> {
        Vec3::zero()
    }

    fn get_surface_area(&self) -> f64 {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }

    fn intersect(&self, ray: Ray<f64>, max_t: f64) -> Option<Interaction> {
        // Now we need to solve the following quadratic equation:
        let a = ray.dir.dot(ray.dir);
        let b = 2. * ray.dir.dot(ray.org);
        let c = ray.org.dot(ray.org) - self.radius * self.radius;

        let (t0, t1) = match quadratic(a, b, c) {
            Some(t) => t,
            _ => return None,
        };

        if t0 > max_t || t1 <= 0. {
            return None;
        }

        let t = if t0 <= 0. { t1 } else { t0 };

        if t > max_t {
            return None;
        }

        // Get the hit point of the intersection in a robust manner:
        let p = ray.org + ray.dir.scale(t);
        let p = p.scale(self.radius / p.length());
        let p = if p.x == 0. && p.y == 0. {
            Vec3 {
                x: 1e-5 * self.radius,
                y: p.y,
                z: p.z,
            }
        } else {
            p
        };

        let phi = p.y.atan2(p.x);
        let phi = if phi < 0. {
            phi + 2. * f64::consts::PI
        } else {
            phi
        };

        // Check against the climping values of the sphere. If this doesn't
        // work, we might have to update the values we just calculated using
        // t1 instead of t0 (if t1 was already being used, we are done):
        let (t, p, phi) = if (self.z_min > -self.radius && p.z < self.z_min)
            || (self.z_max < self.radius && p.z > self.z_max)
            || phi > self.phi_max
        {
            // Make sure that t1 is a valid choice:
            if t == t1 {
                return None;
            }
            if t1 > max_t {
                return None;
            }
            // Calculate p_hit and phi with the new t values here:
            let t = t1;
            let p = ray.org + ray.dir.scale(t);
            let p = p.scale(self.radius / p.length());
            let p = if p.x == 0. && p.y == 0. {
                Vec3 {
                    x: 1e-5 * self.radius,
                    y: p.y,
                    z: p.z,
                }
            } else {
                p
            };

            let phi = p.y.atan2(p.x);
            let phi = if phi < 0. {
                phi + 2. * f64::consts::PI
            } else {
                phi
            };

            if (self.z_min > -self.radius && p.z < self.z_min)
                || (self.z_max < self.radius && p.z > self.z_max)
                || phi > self.phi_max
            {
                return None;
            }

            (t, p, phi)
        // We intersected the correct point:
        } else {
            (t, p, phi)
        };

        // Calculate the u,v coordinates:
        let u = phi / self.phi_max;
        let theta = clamp(p.z / self.radius, -1., 1.).acos();
        let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);

        // Calculate the dpdu and dpdv values:
        let z_radius = (p.x * p.x + p.y * p.y).sqrt();
        let inv_z_radius = 1. / z_radius;
        let cos_phi = p.x * inv_z_radius;
        let sin_phi = p.y * inv_z_radius;
        let dpdu = Vec3 {
            x: -self.phi_max * p.y,
            y: self.phi_max * p.x,
            z: 0.,
        };
        let dpdv = Vec3 {
            x: p.z * cos_phi,
            y: p.z * sin_phi,
            z: -self.radius * theta.sin(),
        }
        .scale(self.theta_max - self.theta_min);

        // Calculate the dndu and dndv values:
        let d2pduu = Vec3 {
            x: p.x,
            y: p.y,
            z: 0.,
        }
        .scale(-self.phi_max * self.phi_max);
        let d2pduv = Vec3 {
            x: -sin_phi,
            y: cos_phi,
            z: 0.,
        }
        .scale((self.theta_max - self.theta_min) * p.z * self.phi_max);
        let d2pdvv =
            p.scale(-(self.theta_max - self.theta_min) * (self.theta_max - self.theta_min));

        // Fundemental forms (see work on diff geometry):
        let be = dpdu.dot(dpdu);
        let bf = dpdu.dot(dpdv);
        let bg = dpdv.dot(dpdv);
        let n = dpdu.cross(dpdv).normalize();
        let e = n.dot(d2pduu);
        let f = n.dot(d2pduv);
        let g = n.dot(d2pdvv);

        // We can now calculate dndu and dndv:
        let inv_begf2 = 1. / (be * bg - bf * bf);
        let dndu = (dpdu.scale(inv_begf2 * (f * bf - e * bg))
            + dpdv.scale(inv_begf2 * (e * bf - f * be)))
        .normalize();
        let dndv = (dpdu.scale(inv_begf2 * (g * bf - f * bg))
            + dpdv.scale(inv_begf2 * (f * bf - g * be)))
        .normalize();

        Some(Interaction {
            p,
            n,
            wo: -ray.dir,
            t,
            uv: Vec2 { x: u, y: v },
            dpdu,
            dpdv,
            // Because it's already a perfect sphere,
            // we just use the same values here:
            shading_n: n,
            shading_dpdu: dpdu,
            shading_dpdv: dpdv,
            shading_dndu: dndu,
            shading_dndv: dndv,
            light: None,
        })
    }

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64) -> bool {
        // Now we need to solve the following quadratic equation:
        let a = ray.dir.dot(ray.dir);
        let b = 2. * ray.dir.dot(ray.org);
        let c = ray.org.dot(ray.org) - self.radius * self.radius;

        let (t0, t1) = match quadratic(a, b, c) {
            Some(t) => t,
            _ => return false,
        };

        if t0 > max_time || t1 <= 0. {
            return false;
        }

        let t = if t0 <= 0. { t1 } else { t0 };

        if t > max_time {
            return false;
        }

        // Get the hit point of the intersection in a robust manner:
        let p = ray.org + ray.dir.scale(t);
        let p = p.scale(self.radius / p.length());
        let p = if p.x == 0. && p.y == 0. {
            Vec3 {
                x: 1e-5 * self.radius,
                y: p.y,
                z: p.z,
            }
        } else {
            p
        };

        let phi = p.y.atan2(p.x);
        let phi = if phi < 0. {
            phi + 2. * f64::consts::PI
        } else {
            phi
        };

        // Check against the climping values of the sphere. If this doesn't
        // work, we might have to update the values we just calculated using
        // t1 instead of t0 (if t1 was already being used, we are done):
        if (self.z_min > -self.radius && p.z < self.z_min)
            || (self.z_max < self.radius && p.z > self.z_max)
            || phi > self.phi_max
        {
            // Make sure that t1 is a valid choice:
            if t == t1 {
                return false;
            }
            if t1 > max_time {
                return false;
            }
            // Calculate p_hit and phi with the new t values here:
            let t = t1;
            let p = ray.org + ray.dir.scale(t);
            let p = p.scale(self.radius / p.length());
            let p = if p.x == 0. && p.y == 0. {
                Vec3 {
                    x: 1e-5 * self.radius,
                    y: p.y,
                    z: p.z,
                }
            } else {
                p
            };

            let phi = p.y.atan2(p.x);
            let phi = if phi < 0. {
                phi + 2. * f64::consts::PI
            } else {
                phi
            };

            if (self.z_min > -self.radius && p.z < self.z_min)
                || (self.z_max < self.radius && p.z > self.z_max)
                || phi > self.phi_max
            {
                return false;
            }
        }

        true
    }
}
