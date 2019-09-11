use crate::math::vector::{Vec3, Vec4};
use crate::math::numbers::Float;

use std::ops::{Add, Index, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Mat4<T: Float> {
    // Array of rows
    m: [Vec4<T>; 4],
}

pub type Mat4f = Mat4<f32>;
pub type Mat4d = Mat4<f64>;

impl<T: Float> Mat4<T> {
    pub fn new(m: [Vec4<T>; 4]) -> Self {
        Mat4 { m }
    }

    // Creates translation matrix:
    pub fn new_translate(trans: Vec3<T>) -> Self {
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

    // Creates a scale matrix:
    pub fn new_scale(scale: Vec3<T>) -> Self {
        let r0 = Vec4 {
            x: scale.x,
            y: T::zero(),
            z: T::zero(),
            w: T::zero(),
        };
        let r1 = Vec4 {
            x: T::zero(),
            y: scale.y,
            z: T::zero(),
            w: T::zero(),
        };
        let r2 = Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: scale.z,
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

    // Creates a rotation matrix:
    pub fn new_rotate(deg: T, axis: Vec3<T>) -> Self {
        let axis = axis.normalize();
        let rad = deg.to_radians();
        let (sin, cos) = rad.sin_cos();

        let r0 = Vec4 {
            x: axis.x * axis.x + (T::one() - axis.x * axis.x) * cos,
            y: axis.x * axis.y * (T::one() - cos) - axis.z * sin,
            z: axis.x * axis.z * (T::one() - cos) + axis.y * sin,
            w: T::zero(),
        };
        let r1 = Vec4 {
            x: axis.x * axis.y * (T::one() - cos) + axis.z * sin,
            y: axis.y * axis.y + (T::one() - axis.y * axis.y) * cos,
            z: axis.y * axis.z * (T::one() - cos) - axis.x * sin,
            w: T::zero(),
        };
        let r2 = Vec4 {
            x: axis.x * axis.z * (T::one() - cos) - axis.y * sin,
            y: axis.y * axis.z * (T::one() - cos) + axis.x * sin,
            z: axis.z * axis.z + (T::one() - axis.z * axis.z) * cos,
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

    // Creates an identity matrix:
    pub fn new_identity() -> Self {
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

    pub fn transpose(self) -> Self {
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

    // returns a column in the matrix:
    pub fn get_column(self, i: usize) -> Vec4<T> {
        Vec4 {
            x: self[0][i],
            y: self[1][i],
            z: self[2][i],
            w: self[3][i],
        }
    }

    // Calculates the inverse of a matrix. Note that, because
    // the inverse can be undefined, it retuns an option.
    pub fn inverse(self) -> Option<Self> {
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
    pub fn mul_vec(self, vec: Vec4<T>) -> Vec4<T> {
        let x = vec.dot(self.m[0]);
        let y = vec.dot(self.m[1]);
        let z = vec.dot(self.m[2]);
        let w = vec.dot(self.m[3]);
        Vec4 { x, y, z, w }
    }

    pub fn scale(self, s: T) -> Self {
        Mat4 {
            m: [
                self.m[0].scale(s),
                self.m[1].scale(s),
                self.m[2].scale(s),
                self.m[3].scale(s),
            ],
        }
    }

    pub fn lerp(self, m1: Self, time: T) -> Self {
        Mat4 {
            m: [
                self[0].lerp(m1[0], time),
                self[1].lerp(m1[1], time),
                self[2].lerp(m1[2], time),
                self[3].lerp(m1[3], time),
            ],
        }
    }
}

impl<T: Float> Index<usize> for Mat4<T> {
    type Output = Vec4<T>;

    // One would have to use [r][c]
    fn index(&self, i: usize) -> &Vec4<T> {
        &self.m[i]
    }
}

impl<T: Float> Neg for Mat4<T> {
    type Output = Mat4<T>;

    fn neg(self) -> Mat4<T> {
        Mat4 {
            m: [-self.m[0], -self.m[1], -self.m[2], -self.m[3]],
        }
    }
}

impl<T: Float> Add for Mat4<T> {
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

impl<T: Float> Sub for Mat4<T> {
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

impl<T: Float> Mul for Mat4<T> {
    type Output = Mat4<T>;

    fn mul(self, o: Mat4<T>) -> Mat4<T> {
        let r0 = Vec4 {
            x: self.m[0].dot(o.m[0]),
            y: self.m[0].dot(o.m[1]),
            z: self.m[0].dot(o.m[2]),
            w: self.m[0].dot(o.m[3]),
        };
        let r1 = Vec4 {
            x: self.m[1].dot(o.m[0]),
            y: self.m[1].dot(o.m[1]),
            z: self.m[1].dot(o.m[2]),
            w: self.m[1].dot(o.m[3]),
        };
        let r2 = Vec4 {
            x: self.m[2].dot(o.m[0]),
            y: self.m[2].dot(o.m[1]),
            z: self.m[2].dot(o.m[2]),
            w: self.m[2].dot(o.m[3]),
        };
        let r3 = Vec4 {
            x: self.m[3].dot(o.m[0]),
            y: self.m[3].dot(o.m[1]),
            z: self.m[3].dot(o.m[2]),
            w: self.m[3].dot(o.m[3]),
        };

        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mat3<T: Float> {
    // Array of rows
    m: [Vec3<T>; 3],
}

pub type Mat3f = Mat3<f32>;
pub type Mat3d = Mat3<f64>;

impl<T: Float> Mat3<T> {
    pub fn new(m: [Vec3<T>; 3]) -> Self {
        Mat3 { m }
    }

    // Extracts the upper 3x3 matrix from a mat4
    pub fn from_mat4(m: Mat4<T>) -> Self {
        let r0 = Vec3 {
            x: m[0][0],
            y: m[1][1],
            z: m[2][2],
        };

        let r1 = Vec3 {
            x: m[1][0],
            y: m[1][1],
            z: m[1][2],
        };

        let r2 = Vec3 {
            x: m[2][0],
            y: m[2][1],
            z: m[2][2],
        };

        Mat3 { m: [r0, r1, r2] }
    }

    // Creates an identity matrix:
    pub fn identity() -> Self {
        let r0 = Vec3 {
            x: T::one(),
            y: T::zero(),
            z: T::zero(),
        };
        let r1 = Vec3 {
            x: T::zero(),
            y: T::one(),
            z: T::zero(),
        };
        let r2 = Vec3 {
            x: T::zero(),
            y: T::zero(),
            z: T::one(),
        };
        Mat3 { m: [r0, r1, r2] }
    }

    // returns a column in the matrix:
    pub fn get_column(self, i: usize) -> Vec3<T> {
        Vec3 {
            x: self[0][i],
            y: self[1][i],
            z: self[2][i],
        }
    }

    pub fn vec_mul(self, vec: Vec3<T>) -> Vec3<T> {
        let x = vec.dot(self.m[0]);
        let y = vec.dot(self.m[1]);
        let z = vec.dot(self.m[2]);
        Vec3 { x, y, z }
    }
}

impl<T: Float> Index<usize> for Mat3<T> {
    type Output = Vec3<T>;

    // One would have to use [r][c]
    fn index(&self, i: usize) -> &Vec3<T> {
        &self.m[i]
    }
}
