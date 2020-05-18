use crate::math::numbers::Float;
use crate::math::vector::{Vec3, Vec4};

use std::ops::{Add, Index, Mul, Neg, Sub};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Mat3x4<T: Float> {
    m: [Vec4<T>; 3], //row-major
}

impl<T: Float> Mat3x4<T> {
    pub fn from_arr(m: [T; 12]) -> Self {
        let r0 = Vec4 {
            x: m[0],
            y: m[1],
            z: m[2],
            w: m[3],
        };
        let r1 = Vec4 {
            x: m[4],
            y: m[5],
            z: m[6],
            w: m[7],
        };
        let r2 = Vec4 {
            x: m[8],
            y: m[9],
            z: m[10],
            w: m[11],
        };
        Mat3x4 { m: [r0, r1, r2] }
    }

    pub fn new(m: [Vec4<T>; 3]) -> Self {
        Mat3x4 { m }
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
        Mat3x4 { m: [r0, r1, r2] }
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
        Mat3x4 { m: [r0, r1, r2] }
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
        Mat3x4 { m: [r0, r1, r2] }
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
        Mat3x4 { m: [r0, r1, r2] }
    }

    // Not a true transpose as the result isn't
    // a 4x3 martix. Can be used for rotations and stuff.
    pub fn transpose(self) -> Self {
        let r0 = Vec4 {
            x: self.m[0].x,
            y: self.m[1].x,
            z: self.m[2].x,
            w: T::zero(), //self.m[3].x,
        };
        let r1 = Vec4 {
            x: self.m[0].y,
            y: self.m[1].y,
            z: self.m[2].y,
            w: T::zero(), //self.m[3].y,
        };
        let r2 = Vec4 {
            x: self.m[0].z,
            y: self.m[1].z,
            z: self.m[2].z,
            w: T::zero(), //self.m[3].z,
        };
        Mat3x4 { m: [r0, r1, r2] }
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

    pub fn determinant(self) -> T {
        let a2323 = self.m[2][2]; // * self.m[3][3] - self.m[2][3] * self.m[3][2];
        let a1323 = self.m[2][1]; // * self.m[3][3] - self.m[2][3] * self.m[3][1];
        let a1223 = T::zero(); // self.m[2][1] * self.m[3][2] - self.m[2][2] * self.m[3][1];
        let a0323 = self.m[2][0]; // * self.m[3][3] - self.m[2][3] * self.m[3][0];
        let a0223 = T::zero(); // self.m[2][0] * self.m[3][2] - self.m[2][2] * self.m[3][0];
        let a0123 = T::zero(); // self.m[2][0] * self.m[3][1] - self.m[2][1] * self.m[3][0];
                               // let a2313 = self.m[1][2]; // * self.m[3][3] - self.m[1][3] * self.m[3][2];
                               // let a1313 = self.m[1][1]; // * self.m[3][3] - self.m[1][3] * self.m[3][1];
                               // let a1213 = T::zero(); // self.m[1][1] * self.m[3][2] - self.m[1][2] * self.m[3][1];
                               // let a2312 = self.m[1][2] * self.m[2][3] - self.m[1][3] * self.m[2][2];
                               // let a1312 = self.m[1][1] * self.m[2][3] - self.m[1][3] * self.m[2][1];
                               // let a1212 = self.m[1][1] * self.m[2][2] - self.m[1][2] * self.m[2][1];
                               // let a0313 = self.m[1][0]; // * self.m[3][3] - self.m[1][3] * self.m[3][0];
                               // let a0213 = T::zero(); // self.m[1][0] * self.m[3][2] - self.m[1][2] * self.m[3][0];
                               // let a0312 = self.m[1][0] * self.m[2][3] - self.m[1][3] * self.m[2][0];
                               // let a0212 = self.m[1][0] * self.m[2][2] - self.m[1][2] * self.m[2][0];
                               // let a0113 = T::zero(); // self.m[1][0] * self.m[3][1] - self.m[1][1] * self.m[3][0];
                               // let a0112 = self.m[1][0] * self.m[2][1] - self.m[1][1] * self.m[2][0];

        let inv_det = self.m[0][0]
            * (self.m[1][1] * a2323 - self.m[1][2] * a1323 + self.m[1][3] * a1223)
            - self.m[0][1] * (self.m[1][0] * a2323 - self.m[1][2] * a0323 + self.m[1][3] * a0223)
            + self.m[0][2] * (self.m[1][0] * a1323 - self.m[1][1] * a0323 + self.m[1][3] * a0123)
            - self.m[0][3] * (self.m[1][0] * a1223 - self.m[1][1] * a0223 + self.m[1][2] * a0123);
        T::one() / inv_det
    }

    pub fn is_invertible(self) -> bool {
        self.determinant() != T::zero()
    }

    // Calculates the inverse of a matrix. It doesn't return a true
    // inverse. It just assumes that the lasat row is (0, 0, 0, 1)
    pub fn inverse(self) -> Self {
        let a2323 = self.m[2][2]; // * self.m[3][3] - self.m[2][3] * self.m[3][2];
        let a1323 = self.m[2][1]; // * self.m[3][3] - self.m[2][3] * self.m[3][1];
        let a1223 = T::zero(); // self.m[2][1] * self.m[3][2] - self.m[2][2] * self.m[3][1];
        let a0323 = self.m[2][0]; // * self.m[3][3] - self.m[2][3] * self.m[3][0];
        let a0223 = T::zero(); // self.m[2][0] * self.m[3][2] - self.m[2][2] * self.m[3][0];
        let a0123 = T::zero(); // self.m[2][0] * self.m[3][1] - self.m[2][1] * self.m[3][0];
        let a2313 = self.m[1][2]; // * self.m[3][3] - self.m[1][3] * self.m[3][2];
        let a1313 = self.m[1][1]; // * self.m[3][3] - self.m[1][3] * self.m[3][1];
        let a1213 = T::zero(); // self.m[1][1] * self.m[3][2] - self.m[1][2] * self.m[3][1];
        let a2312 = self.m[1][2] * self.m[2][3] - self.m[1][3] * self.m[2][2];
        let a1312 = self.m[1][1] * self.m[2][3] - self.m[1][3] * self.m[2][1];
        let a1212 = self.m[1][1] * self.m[2][2] - self.m[1][2] * self.m[2][1];
        let a0313 = self.m[1][0]; // * self.m[3][3] - self.m[1][3] * self.m[3][0];
        let a0213 = T::zero(); // self.m[1][0] * self.m[3][2] - self.m[1][2] * self.m[3][0];
        let a0312 = self.m[1][0] * self.m[2][3] - self.m[1][3] * self.m[2][0];
        let a0212 = self.m[1][0] * self.m[2][2] - self.m[1][2] * self.m[2][0];
        let a0113 = T::zero(); // self.m[1][0] * self.m[3][1] - self.m[1][1] * self.m[3][0];
        let a0112 = self.m[1][0] * self.m[2][1] - self.m[1][1] * self.m[2][0];

        let inv_det = self.m[0][0]
            * (self.m[1][1] * a2323 - self.m[1][2] * a1323 + self.m[1][3] * a1223)
            - self.m[0][1] * (self.m[1][0] * a2323 - self.m[1][2] * a0323 + self.m[1][3] * a0223)
            + self.m[0][2] * (self.m[1][0] * a1323 - self.m[1][1] * a0323 + self.m[1][3] * a0123)
            - self.m[0][3] * (self.m[1][0] * a1223 - self.m[1][1] * a0223 + self.m[1][2] * a0123);
        let det = T::one() / inv_det;

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

        // let r3 = Vec4 {
        //     x: det * -(self.m[1][0] * a1223 - self.m[1][1] * a0223 + self.m[1][2] * a0123),
        //     y: det * (self.m[0][0] * a1223 - self.m[0][1] * a0223 + self.m[0][2] * a0123),
        //     z: det * -(self.m[0][0] * a1213 - self.m[0][1] * a0213 + self.m[0][2] * a0113),
        //     w: det * (self.m[0][0] * a1212 - self.m[0][1] * a0212 + self.m[0][2] * a0112),
        // };

        Mat3x4 { m: [r0, r1, r2] }
    }

    // Multiplies the vector as if it's w component was one
    pub fn mul_vec_one(self, vec: Vec3<T>) -> Vec3<T> {
        let x = self.m[0].dot_one(vec);
        let y = self.m[1].dot_one(vec);
        let z = self.m[2].dot_one(vec);
        Vec3 { x, y, z }
    }

    // Multiplies the vector as if it's w component was a zero
    pub fn mul_vec_zero(self, vec: Vec3<T>) -> Vec3<T> {
        let x = self.m[0].dot_zero(vec);
        let y = self.m[1].dot_zero(vec);
        let z = self.m[2].dot_zero(vec);
        Vec3 { x, y, z }
    }

    pub fn scale(self, s: T) -> Self {
        Mat3x4 {
            m: [self.m[0].scale(s), self.m[1].scale(s), self.m[2].scale(s)],
        }
    }

    pub fn lerp(self, m1: Self, time: T) -> Self {
        Mat3x4 {
            m: [
                self[0].lerp(m1[0], time),
                self[1].lerp(m1[1], time),
                self[2].lerp(m1[2], time),
            ],
        }
    }

    pub fn to_f32(self) -> Mat3x4<f32> {
        Mat3x4 {
            m: [self.m[0].to_f32(), self.m[1].to_f32(), self.m[2].to_f32()],
        }
    }

    pub fn to_f64(self) -> Mat3x4<f64> {
        Mat3x4 {
            m: [self.m[0].to_f64(), self.m[1].to_f64(), self.m[2].to_f64()],
        }
    }
}

impl<T: Float> Index<usize> for Mat3x4<T> {
    type Output = Vec4<T>;

    // One would have to use [r][c]
    fn index(&self, i: usize) -> &Vec4<T> {
        &self.m[i]
    }
}

impl<T: Float> Neg for Mat3x4<T> {
    type Output = Mat3x4<T>;

    fn neg(self) -> Mat3x4<T> {
        Mat3x4 {
            m: [-self.m[0], -self.m[1], -self.m[2]],
        }
    }
}

impl<T: Float> Add for Mat3x4<T> {
    type Output = Mat3x4<T>;

    fn add(self, o: Mat3x4<T>) -> Mat3x4<T> {
        let r0 = self.m[0] + o.m[0];
        let r1 = self.m[1] + o.m[1];
        let r2 = self.m[2] + o.m[2];
        Mat3x4 { m: [r0, r1, r2] }
    }
}

impl<T: Float> Sub for Mat3x4<T> {
    type Output = Mat3x4<T>;

    fn sub(self, o: Mat3x4<T>) -> Mat3x4<T> {
        let r0 = self.m[0] - o.m[0];
        let r1 = self.m[1] - o.m[1];
        let r2 = self.m[2] - o.m[2];
        Mat3x4 { m: [r0, r1, r2] }
    }
}

impl<T: Float> Mul for Mat3x4<T> {
    type Output = Mat3x4<T>;

    // Not a true mulitplication.
    fn mul(self, o: Mat3x4<T>) -> Mat3x4<T> {
        let r0 = Vec4 {
            x: self.m[0].dot(o.m[0]),
            y: self.m[0].dot(o.m[1]),
            z: self.m[0].dot(o.m[2]),
            w: self.m[0].w, //dot(o.m[3]),
        };
        let r1 = Vec4 {
            x: self.m[1].dot(o.m[0]),
            y: self.m[1].dot(o.m[1]),
            z: self.m[1].dot(o.m[2]),
            w: self.m[1].w, //.dot(o.m[3]),
        };
        let r2 = Vec4 {
            x: self.m[2].dot(o.m[0]),
            y: self.m[2].dot(o.m[1]),
            z: self.m[2].dot(o.m[2]),
            w: self.m[2].w, //.dot(o.m[3]),
        };

        Mat3x4 { m: [r0, r1, r2] }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Mat4<T: Float> {
    m: [Vec4<T>; 4],
}

impl<T: Float> Mat4<T> {
    pub fn from_mat3x4(m: Mat3x4<T>) -> Self {
        Mat4 {
            m: [
                m[0],
                m[1],
                m[2],
                Vec4 {
                    x: T::zero(),
                    y: T::zero(),
                    z: T::zero(),
                    w: T::one(),
                },
            ],
        }
    }

    pub fn from_arr(m: [T; 16]) -> Self {
        let r0 = Vec4 {
            x: m[0],
            y: m[1],
            z: m[2],
            w: m[3],
        };
        let r1 = Vec4 {
            x: m[4],
            y: m[5],
            z: m[6],
            w: m[7],
        };
        let r2 = Vec4 {
            x: m[8],
            y: m[9],
            z: m[10],
            w: m[11],
        };
        let r3 = Vec4 {
            x: m[12],
            y: m[13],
            z: m[14],
            w: m[15],
        };
        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }

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

    pub fn new_perspective(fov: T, near: T, far: T) -> Self {
        let perspective = Mat4::new([
            Vec4 {
                x: T::one(),
                y: T::zero(),
                z: T::zero(),
                w: T::zero(),
            },
            Vec4 {
                x: T::zero(),
                y: T::one(),
                z: T::zero(),
                w: T::zero(),
            },
            Vec4 {
                x: T::zero(),
                y: T::zero(),
                z: far / (far - near),
                w: -far * near / (far - near),
            },
            Vec4 {
                x: T::zero(),
                y: T::zero(),
                z: T::one(),
                w: T::zero(),
            },
        ]);
        // Calculate the FOV information:
        let inv_tan_angle = T::one() / (fov.to_radians() / (T::one() + T::one())).tan();
        // Calculate the scale used:
        let scale_mat = Self::new_scale(Vec3 {
            x: inv_tan_angle,
            y: inv_tan_angle,
            z: T::one(),
        });

        scale_mat * perspective
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
    pub fn inverse(self) -> Self {
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

        Mat4 {
            m: [r0, r1, r2, r3],
        }
    }

    /// Performs a matrix multiplication with a vector in a generic manner:
    pub fn mul_vec(self, vec: Vec4<T>) -> Vec4<T> {
        let x = vec.dot(self.m[0]);
        let y = vec.dot(self.m[1]);
        let z = vec.dot(self.m[2]);
        let w = vec.dot(self.m[3]);
        Vec4 { x, y, z, w }
    }

    pub fn mul_vec_one(self, vec: Vec3<T>) -> Vec3<T> {
        let x = self.m[0].dot_one(vec);
        let y = self.m[1].dot_one(vec);
        let z = self.m[2].dot_one(vec);
        Vec3 { x, y, z }
    }

    pub fn mul_vec_zero(self, vec: Vec3<T>) -> Vec3<T> {
        let x = self.m[0].dot_zero(vec);
        let y = self.m[1].dot_zero(vec);
        let z = self.m[2].dot_zero(vec);
        Vec3 { x, y, z }
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

#[repr(C)]
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

    pub fn from_mat3x4(m: Mat3x4<T>) -> Self {
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
