// This holds traits that are used throughout the program
// to make things easy for us:

use num_traits;

// This is my own float trait versus the num_traits' one:

pub trait Float: num_traits::Float + num_traits::Bounded {
    const PI: Self;
    const INV_PI: Self;
    const INV_2PI: Self;
    const INV_4PI: Self;
    const PI_OVER_2: Self;
    const PI_OVER_4: Self;
    const SQRT_2: Self;
    const ONE_MINUS_EPS: Self;

    // Special constants:
    const SELF_INT_COMP: Self; // small offset used to compensate for self intersections

    // Some important functions that are lacking from the previous one:

    // These could be constants, but to mimick the design of num_traits' float
    // we'll make them functions:
    fn two() -> Self;
    fn half() -> Self;

    fn to_f32(self) -> f32;
    fn to_f64(self) -> f64;
}

impl Float for f32 {
    const PI: f32 = 3.14159265358979323846;
    const INV_PI: f32 = 0.31830988618379067154;
    const INV_2PI: f32 = 0.15915494309189533577;
    const INV_4PI: f32 = 0.07957747154594766788;
    const PI_OVER_2: f32 = 1.57079632679489661923;
    const PI_OVER_4: f32 = 0.78539816339744830961;
    const SQRT_2: f32 = 1.41421356237309504880;
    const ONE_MINUS_EPS: f32 = 0.99999994;
    const SELF_INT_COMP: f32 = 0.00001;

    fn two() -> f32 {
        2f32
    }

    fn half() -> f32 {
        0.5f32
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl Float for f64 {
    const PI: f64 = 3.14159265358979323846;
    const INV_PI: f64 = 0.31830988618379067154;
    const INV_2PI: f64 = 0.15915494309189533577;
    const INV_4PI: f64 = 0.07957747154594766788;
    const PI_OVER_2: f64 = 1.57079632679489661923;
    const PI_OVER_4: f64 = 0.78539816339744830961;
    const SQRT_2: f64 = 1.41421356237309504880;
    const ONE_MINUS_EPS: f64 = 0.99999999999999989;
    const SELF_INT_COMP: f64 = 0.00001;

    fn two() -> f64 {
        2.
    }

    fn half() -> f64 {
        0.5
    }

    fn to_f32(self) -> f32 {
        self as f32
    }

    fn to_f64(self) -> f64 {
        self
    }
}
