use crate::math::matrix::Mat4;
use crate::math::vector::{Vec3, Vec4};

use num_traits::{clamp, Float};

use std::mem::MaybeUninit;
use std::ops::{Add, Mul, Sub, Neg};

#[derive(Clone, Copy, Debug)]
pub struct Quat<T: Float> {
    pub xyz: Vec3<T>,
    pub w: T,
}

impl<T: Float> Quat<T> {
    pub fn from_mat4(mat: Mat4<T>) -> Self {
        let tr = mat[0][0] + mat[1][1] + mat[2][2];

        let two = T::from::<f64>(2.0).unwrap();
        let half = T::from::<f64>(0.5).unwrap();

        if tr > T::zero() {
            let s = (tr + T::one()).sqrt();
            let w = s / two;
            let xyz = Vec3 {
                x: (mat[2][1] - mat[1][2]) * s,
                y: (mat[0][2] - mat[2][0]) * s,
                z: (mat[1][0] - mat[0][1]) * s,
            };

            Quat { xyz, w }
        } else {
            let (i, j, k) = if mat[0][0] > mat[1][1] && mat[0][0] > mat[2][2] {
                (0, 1, 2)
            } else if mat[1][1] > mat[2][2] {
                (1, 2, 0)
            } else {
                (2, 0, 1)
            };

            let mut xyz: Vec3<T> = unsafe { MaybeUninit::uninit().assume_init() };

            let s = (mat[i][i] - (mat[j][j] + mat[k][k]) + T::one()).sqrt();
            xyz[i] = s * half;
            let s = if s != T::zero() { half / s } else { s };
            xyz[j] = s * (mat[j][i] + mat[i][j]);
            xyz[k] = s * (mat[k][i] + mat[i][k]);

            let w = s * (mat[k][j] - mat[j][k]);

            Quat { xyz, w }
        }
    }

    pub fn to_mat4(self) -> Mat4<T> {
        let x2 = self.xyz.x * self.xyz.x;
        let y2 = self.xyz.y * self.xyz.y;
        let z2 = self.xyz.z * self.xyz.z;

        let xy = self.xyz.x * self.xyz.y;
        let xz = self.xyz.x * self.xyz.z;
        let yz = self.xyz.y * self.xyz.z;

        let wx = self.xyz.x * self.w;
        let wy = self.xyz.y * self.w;
        let wz = self.xyz.z * self.w;

        // I know this is dumb, I'll figure something out for
        // this later:
        let two = T::from::<f64>(2.0).unwrap();

        let r0 = Vec4 {
            x: T::one() - two * (y2 + z2),
            y: two * (xy + wz),
            z: two * (xz - wy),
            w: T::zero(),
        };

        let r1 = Vec4 {
            x: two * (xy - wz),
            y: T::one() - two * (x2 + z2),
            z: two * (yz + wx),
            w: T::zero(),
        };

        let r2 = Vec4 {
            x: two * (xz + wy),
            y: two * (yz - wx),
            z: T::one() - two * (x2 + y2),
            w: T::zero(),
        };

        let r3 = Vec4 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
            w: T::one(),
        };

        Mat4::new([r0, r1, r2, r3])
    }

    pub fn slerp(self, q2: Self, t: T) -> Self {
        let cos_theta = self.dot(q2);
        // This constant is used in pbrt. So I'll just use it here:
        if cos_theta > T::from::<f32>(0.9995f32).unwrap() {
            (self.scale(T::one() - t) + q2.scale(t)).normalize()
        } else {
            let theta = clamp(cos_theta, -T::one(), T::one()).acos();
            let theta_p = theta * t;
            let q_perp = (q2 - self.scale(cos_theta)).normalize();

            self.scale(theta_p.cos()) + q_perp.scale(theta_p.sin())
        }
    }

    pub fn dot(self, o: Self) -> T {
        self.xyz.dot(o.xyz) + self.w * o.w
    }

    pub fn length2(self) -> T {
        self.dot(self)
    }

    pub fn length(self) -> T {
        self.length2().sqrt()
    }

    pub fn normalize(self) -> Self {
        let inv_len = T::one() / self.length();
        self.scale(inv_len)
    }

    pub fn scale(self, s: T) -> Self {
        Quat {
            xyz: self.xyz.scale(s),
            w: self.w * s,
        }
    }
}

impl<T: Float> Neg for Quat<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Quat {
            xyz: -self.xyz,
            w: -self.w,
        }
    }
}

impl<T: Float> Mul for Quat<T> {
    type Output = Self;

    fn mul(self, o: Quat<T>) -> Self {
        Quat {
            xyz: self.xyz.cross(o.xyz) + o.xyz.scale(self.w) + self.xyz.scale(o.w),
            w: self.w * o.w - self.xyz.dot(o.xyz),
        }
    }
}

impl<T: Float> Add for Quat<T> {
    type Output = Self;

    fn add(self, o: Quat<T>) -> Self {
        Quat {
            xyz: self.xyz + o.xyz,
            w: self.w + o.w,
        }
    }
}

impl<T: Float> Sub for Quat<T> {
    type Output = Self;

    fn sub(self, o: Quat<T>) -> Self {
        Quat {
            xyz: self.xyz - o.xyz,
            w: self.w - o.w,
        }
    }
}