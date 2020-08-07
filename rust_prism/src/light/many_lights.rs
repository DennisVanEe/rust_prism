// This code is based on the paper from:
// Importance Sampling of Many Lights with Adaptive Tree Splitting by
// Estevez and Kulla.

use pmath::matrix::Mat4;
use pmath::numbers::Float;
use pmath::vector::Vec3;

#[derive(Copy, Clone, Debug)]
struct Cone {
    axis: Vec3<f64>,
    theta_o: f64, // All angles are in radians
    theta_e: f64,
}

impl Cone {
    /// Combines two `Cone`s into one `Cone` that encompasses everything.
    fn union(self, b: Cone) -> Self {
        // Ensure that a.theta_o > b.theta_o
        let (a, b) = if self.theta_o > b.theta_o {
            (self, b)
        } else {
            (b, self)
        };

        let theta_d = a.axis.dot(b.axis).acos();
        let theta_e = a.theta_e.max(b.theta_e);

        if f64::PI.min(theta_d + b.theta_o) <= a.theta_o {
            return Cone {
                axis: a.axis,
                theta_o: a.theta_o,
                theta_e,
            };
        }

        let theta_o = (a.theta_o + theta_d + b.theta_o) * 0.5;
        if f64::PI <= theta_o {
            return Cone {
                axis: a.axis,
                theta_o: f64::PI,
                theta_e,
            };
        }

        let theta_r = theta_o - a.theta_o;
        let axis = {
            // Create a rotation matrix around a.axis x b.axis:
            let rot_mat = Mat4::new_rotate(theta_r.to_degrees(), a.axis.cross(b.axis));
            rot_mat.mul_vec_zero(a.axis)
        };

        Cone {
            axis,
            theta_o,
            theta_e,
        }
    }
}
