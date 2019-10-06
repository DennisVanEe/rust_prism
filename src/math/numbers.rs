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

    // Some important functions that are lacking from the previous one:

    // These could be constants, but to mimick the design of num_traits' float
    // we'll make them functions:
    fn two() -> Self;
    fn half() -> Self;
}

impl Float for f32 {
    const PI: Self = 3.14159265358979323846;
    const INV_PI: Self = 0.31830988618379067154;
    const INV_2PI: Self = 0.15915494309189533577;
    const INV_4PI: Self = 0.07957747154594766788;
    const PI_OVER_2: Self = 1.57079632679489661923;
    const PI_OVER_4: Self = 0.78539816339744830961;
    const SQRT_2: Self = 1.41421356237309504880;
    const ONE_MINUS_EPS: Self = 0.99999994;

    fn two() -> Self {
        2f32
    }

    fn half() -> Self {
        0.5f32
    }
}

impl Float for f64 {
    const PI: Self = 3.14159265358979323846;
    const INV_PI: Self = 0.31830988618379067154;
    const INV_2PI: Self = 0.15915494309189533577;
    const INV_4PI: Self = 0.07957747154594766788;
    const PI_OVER_2: Self = 1.57079632679489661923;
    const PI_OVER_4: Self = 0.78539816339744830961;
    const SQRT_2: Self = 1.41421356237309504880;
    const ONE_MINUS_EPS: Self = 0.99999999999999989;

    fn two() -> Self {
        2.
    }

    fn half() -> Self {
        0.5
    }
}
