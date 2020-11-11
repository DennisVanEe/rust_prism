use crate::interaction::{GeomIntr, Interaction, IntrType, VolIntr};
use pmath::bbox::BBox3;
use pmath::matrix::{Mat3x4, Mat4};
use pmath::ray::Ray;
use pmath::vector::{Vec3, Vec4};

use std::ops::Mul;

#[derive(Clone, Copy, Debug)]
pub struct Transf {
    frd: Mat3x4<f64>,
    inv: Mat3x4<f64>,
}

impl Transf {
    pub fn from_mat4(mat: Mat4<f64>) -> Self {
        let frd = Mat3x4::from_mat4(mat);
        Transf {
            frd,
            inv: frd.inverse(),
        }
    }

    pub fn from_mat3x4(mat: Mat3x4<f64>) -> Self {
        Transf {
            frd: mat,
            inv: mat.inverse(),
        }
    }

    pub fn new_identity() -> Self {
        Transf {
            frd: Mat3x4::new_identity(),
            inv: Mat3x4::new_identity(),
        }
    }

    pub fn new_translate(trans: Vec3<f64>) -> Self {
        Transf {
            frd: Mat3x4::new_translate(trans),
            inv: Mat3x4::new_translate(-trans),
        }
    }

    pub fn new_scale(scale: Vec3<f64>) -> Self {
        Transf {
            frd: Mat3x4::new_scale(scale),
            inv: Mat3x4::new_scale(scale.inv_scale(1.)),
        }
    }

    pub fn new_rotate(deg: f64, axis: Vec3<f64>) -> Self {
        let frd = Mat3x4::new_rotate(deg, axis);
        // inverse of rotation matrix is transpose
        Transf {
            frd,
            inv: frd.transpose(),
        }
    }

    /// Creats a lookat transformation. This is a transformation that goes from
    /// camera to world space
    ///
    /// Note: camera space has the positive z-axis go into the screen, the y-axis pointing
    /// up, and the x-axis pointing right (it's a left-handed coordinate system).
    pub fn new_lookat(up: Vec3<f64>, at: Vec3<f64>, pos: Vec3<f64>) -> Self {
        let f = (at - pos).normalize();
        let s = up.cross(f).normalize();
        let u = f.cross(s);

        let r0 = Vec4::from_vec3(s, -s.dot(pos));
        let r1 = Vec4::from_vec3(u, -u.dot(pos));
        let r2 = Vec4::from_vec3(f, -f.dot(pos));

        let inv = Mat3x4::from_rows([r0, r1, r2]);

        Transf {
            frd: inv.inverse(),
            inv,
        }
    }

    /// Inverses the transformation
    pub fn inverse(&self) -> Self {
        Transf {
            frd: self.inv,
            inv: self.frd,
        }
    }

    // Returns the normal matrix:
    pub fn get_frd(self) -> Mat3x4<f64> {
        self.frd
    }

    pub fn get_inv(self) -> Mat3x4<f64> {
        self.inv
    }

    pub fn point(self, p: Vec3<f64>) -> Vec3<f64> {
        self.frd.mul_vec_one(p)
    }

    pub fn points(self, ps: &mut [Vec3<f64>]) {
        for p in ps.iter_mut() {
            *p = self.point(*p);
        }
    }

    pub fn points_f32(self, ps: &mut [Vec3<f32>]) {
        for p in ps.iter_mut() {
            *p = self.point(p.to_f64()).to_f32();
        }
    }

    pub fn normal(self, n: Vec3<f64>) -> Vec3<f64> {
        self.inv.transpose().mul_vec_zero(n)
    }

    pub fn normals(self, ns: &mut [Vec3<f64>]) {
        let mat = self.inv.transpose();
        for n in ns.iter_mut() {
            *n = mat.mul_vec_zero(*n);
        }
    }

    pub fn normals_f32(self, ns: &mut [Vec3<f32>]) {
        let mat = self.inv.transpose();
        for n in ns.iter_mut() {
            *n = mat.mul_vec_zero(n.to_f64()).to_f32();
        }
    }

    pub fn vector(self, v: Vec3<f64>) -> Vec3<f64> {
        self.frd.mul_vec_zero(v)
    }

    pub fn vectors(self, vs: &mut [Vec3<f64>]) {
        for v in vs.iter_mut() {
            *v = self.frd.mul_vec_zero(*v);
        }
    }

    pub fn vectors_f32(self, vs: &mut [Vec3<f32>]) {
        for v in vs.iter_mut() {
            *v = self.point(v.to_f64()).to_f32();
        }
    }

    pub fn bbox(&self, b: BBox3<f64>) -> BBox3<f64> {
        // From Arvo 1990 Graphics Gems 1

        let pmin = self.frd.get_column(3);
        let pmax = pmin;

        let a0 = self.frd.get_column(0) * b.pmin;
        let a1 = self.frd.get_column(0) * b.pmax;
        let pmin = pmin + a0.min(a1);
        let pmax = pmax + a0.max(a1);

        let a0 = self.frd.get_column(1) * b.pmin;
        let a1 = self.frd.get_column(1) * b.pmax;
        let pmin = pmin + a0.min(a1);
        let pmax = pmax + a0.max(a1);

        let a0 = self.frd.get_column(2) * b.pmin;
        let a1 = self.frd.get_column(2) * b.pmax;
        let pmin = pmin + a0.min(a1);
        let pmax = pmax + a0.max(a1);

        BBox3 { pmin, pmax }
    }

    pub fn interaction(self, i: Interaction) -> Interaction {
        Interaction {
            p: self.point(i.p),
            n: self.normal(i.n).normalize(),
            wo: self.vector(i.wo).normalize(),
            t: i.t,
            time: i.time,

            intr_type: match i.intr_type {
                IntrType::Geom(geom_intr) => IntrType::Geom(self.geom_intr(geom_intr)),
                IntrType::Vol(vol_intr) => IntrType::Vol(self.vol_intr(vol_intr)),
            },
        }
    }

    pub fn geom_intr(self, g: GeomIntr) -> GeomIntr {
        GeomIntr {
            uv: g.uv,
            dpdu: self.vector(g.dpdu),
            dpdv: self.vector(g.dpdv),

            sn: self.normal(g.sn).normalize(),
            sdpdu: self.vector(g.sdpdu),
            sdpdv: self.vector(g.sdpdv),
            sdndu: self.normal(g.sdndu),
            sdndv: self.normal(g.sdndv),
        }
    }

    pub fn vol_intr(self, v: VolIntr) -> VolIntr {
        v
    }

    pub fn ray(self, r: Ray<f64>) -> Ray<f64> {
        Ray {
            org: self.point(r.org),
            dir: self.vector(r.dir),
            time: r.time,
            t_far: r.t_far,
            t_near: r.t_near,
        }
    }
}

impl Mul for Transf {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Transf {
            frd: self.frd * rhs.frd,
            inv: rhs.inv * self.inv,
        }
    }
}
