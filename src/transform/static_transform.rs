// A static transformation is one that is independent of time.
// So, regardless of the time that is passed to it, it doesn't
// Change it's transformation.

use crate::transform::Transform;
use crate::math::matrix::{Mat3, Mat4};
use crate::math::vector::{Vec3, Vec4};
use crate::math::ray::Ray;
use crate::math::bbox::BBox3;

#[derive(Clone, Copy)]
pub struct StaticTransform {
    mat: Mat4<f64>,
    inv: Mat4<f64>,
}

impl StaticTransform {
    // Because a matrix could potentially not be invertible,
    // there is no gaurantee this will work:
    pub fn new(mat: Mat4<f64>) -> Option<Self> {
        let inv = match mat.inverse() {
            Some(i) => i,
            _ => return None,
        };
        Some(StaticTransform { mat, inv })
    }

    // Create a StaticTransformation from a bunch of common ones:

    pub fn new_identity() -> Self {
        StaticTransform {
            mat: Mat4::new_identity(),
            inv: Mat4::new_identity(),
        }
    }

    pub fn new_translate(trans: Vec3<f64>) -> Self {
        StaticTransform { 
            mat: Mat4::new_translate(trans), 
            inv: Mat4::new_translate(-trans) 
        }
    }

    pub fn new_scale(scale: Vec3<f64>) -> Self {
        StaticTransform {
             mat: Mat4::new_scale(scale), 
             inv: Mat4::new_scale(scale.inv_scale(T::one())) 
        }
    }

    pub fn new_rotate(deg: f64, axis: Vec3<f64>) -> Self {
        let mat = Mat4::new_rotate(deg, axis);
        // inverse of rotation matrix is transpose
        let inv = mat.transpose();
        StaticTransform { mat, inv }
    }
}

impl Transform for StaticTransform {
    fn inverse(&self) -> Self {
        StaticTransform { mat: self.inv, inv: self.mat }
    }

    fn point(&self, p: Vec3<f64>, t: f64) -> Vec3<f64> {
        let homog_p = Vec4::from_vec3(p, 1.);
        let homog_r = self.mat.mul_vec(homog_p);
        Vec3::from_vec4(homog_r)
    }

    fn normal(&self, n: Vec3<f64>, t: f64) -> Vec3<f64> {
        let homog_n = Vec4::from_vec3(n, 0.);
        let homog_r = self.inv.transpose().mul_vec(homog_n);
        Vec3::from_vec4(homog_r)
    }

    fn vector(&self, v: Vec3<f64>, t: f64) -> Vec3<f64> {
        let homog_v = Vec4::from_vec3(v, 0.);
        let homog_r = self.mat.mul_vec(homog_v);
        Vec3::from_vec4(homog_r)
    }

    fn ray(&self, r: Ray<f64>, t: f64) -> Ray<f64> {
        Ray {
            org: self.point(r.org, t),
            dir: self.vector(r.dir, t),
        }
    }

    fn bbox(&self, b: BBox3<f64>, t: f64) -> BBox3<f64> {
        // From Arvo 1990 Graphics Gems 1 

        let pmin = Vec3::from_vec4(self.mat.get_column(3));
        let pmax = pmin;

        let rot = Mat3::from_mat4(self.mat);

        let a = rot.get_column(0) * bbox.pmin;
        let b = rot.get_column(0) * bbox.pmax;
        let pmin = pmin + a.min(b);
        let pmax = pmax + a.max(b);

        let a = rot.get_column(1) * bbox.pmin;
        let b = rot.get_column(1) * bbox.pmax;
        let pmin = pmin + a.min(b);
        let pmax = pmax + a.max(b);

        let a = rot.get_column(2) * bbox.pmin;
        let b = rot.get_column(2) * bbox.pmax;
        let pmin = pmin + a.min(b);
        let pmax = pmax + a.max(b);

        BBox3 { pmin, pmax }
    }
}