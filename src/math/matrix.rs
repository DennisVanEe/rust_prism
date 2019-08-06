use num_traits::{Float, Signed};

use std::ops::{Add, Index, Mul, Neg, Sub};

use super::vector::Vec3;
use super::vector::Vec4;

// Not copyable, as Matrices are expensive.
#[derive(Clone, Copy, Debug)]
pub struct Mat4<T: Signed + Float> {
    m: [Vec4<T>; 4],
}

impl<T: Signed + Float> Mat4<T> {
    pub fn transpose(&self) -> Mat4<T> {
        let r0 = Vec4 {
            x: self.m[0].x,
            y: self.m[1].x,
            z: self.m[2].x,
            w: self.m[3].x,
        };
        let r1 = Vec4 {
            x: self.m[0].y,
            y: self.m[1].y,
            z: self.m[2].y,
            w: self.m[3].y,
        };
        let r2 = Vec4 {
            x: self.m[0].z,
            y: self.m[1].z,
            z: self.m[2].z,
            w: self.m[3].z,
        };
        let r3 = Vec4 {
            x: self.m[0].w,
            y: self.m[1].w,
            z: self.m[2].w,
            w: self.m[3].w,
        };
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }

    /// Calculates the inverse of a matrix. Note that, because
    /// the inverse can be undefined, it retuns an option.
    pub fn inverse(&self) -> Option<Mat4<T>> {
        let a2323 = self.m[2][2] * self.m[3][3] - self.m[2][3] * self.m[3][2];
        let a1323 = self.m[2][1] * self.m[3][3] - self.m[2][3] * self.m[3][1];
        let a1223 = self.m[2][1] * self.m[3][2] - self.m[2][2] * self.m[3][1];
        let a0323 = self.m[2][0] * self.m[3][3] - self.m[2][3] * self.m[3][0];
        let a0223 = self.m[2][0] * self.m[3][2] - self.m[2][2] * self.m[3][0];
        let a0123 = self.m[2][0] * self.m[3][1] - self.m[2][1] * self.m[3][0];
        let a2313 = self.m[1][2] * self.m[3][3] - self.m[1][3] * self.m[3][2];
        let a1313 = self.m[1][1] * self.m[3][3] - self.m[1][3] * self.m[3][1];
        let a1213 = self.m[1][1] * self.m[3][2] - self.m[1][2] * self.m[3][1];
        let a2312 = self.m[1][2] * self.m[2][3] - self.m[1][3] * self.m[2][2];
        let a1312 = self.m[1][1] * self.m[2][3] - self.m[1][3] * self.m[2][1];
        let a1212 = self.m[1][1] * self.m[2][2] - self.m[1][2] * self.m[2][1];
        let a0313 = self.m[1][0] * self.m[3][3] - self.m[1][3] * self.m[3][0];
        let a0213 = self.m[1][0] * self.m[3][2] - self.m[1][2] * self.m[3][0];
        let a0312 = self.m[1][0] * self.m[2][3] - self.m[1][3] * self.m[2][0];
        let a0212 = self.m[1][0] * self.m[2][2] - self.m[1][2] * self.m[2][0];
        let a0113 = self.m[1][0] * self.m[3][1] - self.m[1][1] * self.m[3][0];
        let a0112 = self.m[1][0] * self.m[2][1] - self.m[1][1] * self.m[2][0];

        let inv_det = self.m[0][0]
            * (self.m[1][1] * a2323 - self.m[1][2] * a1323 + self.m[1][3] * a1223)
            - self.m[0][1] * (self.m[1][0] * a2323 - self.m[1][2] * a0323 + self.m[1][3] * a0223)
            + self.m[0][2] * (self.m[1][0] * a1323 - self.m[1][1] * a0323 + self.m[1][3] * a0123)
            - self.m[0][3] * (self.m[1][0] * a1223 - self.m[1][1] * a0223 + self.m[1][2] * a0123);
        let det = T::one() / inv_det;

        // Check if the determinant is zero (might have to do this another way later):
        if det == T::zero() {
            None
        } else {
            let r0 = Vec4 {
                x: det * (self.m[1][1] * a2323 - self.m[1][2] * a1323 + self.m[1][3] * a1223),
                y: det * -(self.m[0][1] * a2323 - self.m[0][2] * a1323 + self.m[0][3] * a1223),
                z: det * (self.m[0][1] * a2313 - self.m[0][2] * a1313 + self.m[0][3] * a1213),
                w: det * -(self.m[0][1] * a2312 - self.m[0][2] * a1312 + self.m[0][3] * a1212),
            };

            let r1 = Vec4 {
                x: det * -(self.m[1][0] * a2323 - self.m[1][2] * a0323 + self.m[1][3] * a0223),
                y: det * (self.m[0][0] * a2323 - self.m[0][2] * a0323 + self.m[0][3] * a0223),
                z: det * -(self.m[0][0] * a2313 - self.m[0][2] * a0313 + self.m[0][3] * a0213),
                w: det * (self.m[0][0] * a2312 - self.m[0][2] * a0312 + self.m[0][3] * a0212),
            };

            let r2 = Vec4 {
                x: det * (self.m[1][0] * a1323 - self.m[1][1] * a0323 + self.m[1][3] * a0123),
                y: det * -(self.m[0][0] * a1323 - self.m[0][1] * a0323 + self.m[0][3] * a0123),
                z: det * (self.m[0][0] * a1313 - self.m[0][1] * a0313 + self.m[0][3] * a0113),
                w: det * -(self.m[0][0] * a1312 - self.m[0][1] * a0312 + self.m[0][3] * a0112),
            };

            let r3 = Vec4 {
                x: det * -(self.m[1][0] * a1223 - self.m[1][1] * a0223 + self.m[1][2] * a0123),
                y: det * (self.m[0][0] * a1223 - self.m[0][1] * a0223 + self.m[0][2] * a0123),
                z: det * -(self.m[0][0] * a1213 - self.m[0][1] * a0213 + self.m[0][2] * a0113),
                w: det * (self.m[0][0] * a1212 - self.m[0][1] * a0212 + self.m[0][2] * a0112),
            };

            Some(Mat4 {
                m: [r0, r1, r2, r3],
            })
        }
    }

    /// Performs a matrix multiplication with a vector:
    pub fn vec_mul(&self, vec: Vec4<T>) -> Vec4<T> {
        let x = vec.dot(self.m[0]);
        let y = vec.dot(self.m[1]);
        let z = vec.dot(self.m[2]);
        let w = vec.dot(self.m[3]);
        Vec4 { x, y, z, w }
    }
}

impl<T: Signed + Float> Index<usize> for Mat4<T> {
    type Output = Vec4<T>;

    // One would have to use [r][c]
    fn index(&self, i: usize) -> &Vec4<T> {
        &self.m[i]
    }
}

impl<T: Signed + Float> Neg for Mat4<T> {
    type Output = Mat4<T>;

    fn neg(self) -> Mat4<T> {
        Mat4 {
            m: [-self.m[0], -self.m[1], -self.m[2], -self.m[3]],
        }
    }
}

impl<T: Signed + Float> Add for Mat4<T> {
    type Output = Mat4<T>;

    fn add(self, o: Mat4<T>) -> Mat4<T> {
        let r0 = self.m[0] + o.m[0];
        let r1 = self.m[1] + o.m[1];
        let r2 = self.m[2] + o.m[2];
        let r3 = self.m[3] + o.m[3];
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }
}

impl<T: Signed + Float> Sub for Mat4<T> {
    type Output = Mat4<T>;

    fn sub(self, o: Mat4<T>) -> Mat4<T> {
        let r0 = self.m[0] - o.m[0];
        let r1 = self.m[1] - o.m[1];
        let r2 = self.m[2] - o.m[2];
        let r3 = self.m[3] - o.m[3];
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }
}

impl<T: Signed + Float> Mul for Mat4<T> {
    type Output = Mat4<T>;

    fn mul(self, o: Mat4<T>) -> Mat4<T> {
        let r0 = self.m[0] - o.m[0];
        let r1 = self.m[1] - o.m[1];
        let r2 = self.m[2] - o.m[2];
        let r3 = self.m[3] - o.m[3];
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }
}

impl<T: Signed + Float> Mat4<T> {
    pub fn at(&self, r: usize, c: usize) -> &T {
        &self.m[r][c]
    }

    // Creates an identity matrix:
    pub fn identity() -> Mat4<T> {
        let r0 = Vec4 {
            x: T::one(),
            y: T::zero(),
            z: T::zero(),
            w: T::zero(),
        };
        let r1 = Vec4 {
            x: T::zero(),
            y: T::one(),
            z: T::zero(),
            w: T::zero(),
        };
        let r2 = Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: T::one(),
            w: T::zero(),
        };
        let r3 = Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
            w: T::one(),
        };
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }

    // Creates translation matrix:
    pub fn translation(trans: Vec3<T>) -> Mat4<T> {
        let r0 = Vec4 {
            x: T::one(),
            y: T::zero(),
            z: T::zero(),
            w: trans.x,
        };
        let r1 = Vec4 {
            x: T::zero(),
            y: T::one(),
            z: T::zero(),
            w: trans.y,
        };
        let r2 = Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: T::one(),
            w: trans.z,
        };
        let r3 = Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
            w: T::one(),
        };
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }
}
