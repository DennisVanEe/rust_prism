// This code is based on the paper from:
// Importance Sampling of Many Lights with Adaptive Tree Splitting by
// Estevez and Kulla.

use crate::light::Light;
use bumpalo::Bump;
use partition;
use pmath::bbox::BBox3;
use pmath::matrix::Mat3x4;
use pmath::numbers::Float;
use pmath::vector::Vec3;

/// A light cone represents the extent of a light
#[derive(Copy, Clone, Debug)]
pub struct Cone {
    axis: Vec3<f64>,
    theta_o: f64, // All angles are in radians
    theta_e: f64,
}

impl Cone {
    /// Construct an "initial" `LightCone`. This cone will equal whatever cone ii is combined with.
    pub fn new_initial() -> Self {
        Cone {
            axis: Vec3::zero(),
            theta_o: 0.0,
            theta_e: 0.0,
        }
    }

    /// Construct a new `LightCone`:
    pub fn new(axis: Vec3<f64>, theta_o: f64, theta_e: f64) -> Self {
        Cone {
            axis: axis.normalize(),
            theta_o: theta_o.max(0.0).min(f64::PI), // Make sure theta_o is in [0, PI]
            theta_e: theta_e.max(0.0).min(f64::PI_OVER_2), // Make sure theta_e is in [0, PI/2]
        }
    }

    /// Combines two `LightCone`s into one `LightCone` that encompasses everything.
    fn combine(self, b: Cone) -> Self {
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
            let rot_mat = Mat3x4::new_rotate(theta_r.to_degrees(), a.axis.cross(b.axis));
            rot_mat.mul_vec_zero(a.axis)
        };

        Cone {
            axis,
            theta_o,
            theta_e,
        }
    }

    fn surface_area_orientation_heuristic(self) -> f64 {
        let theta_w = f64::PI.min(self.theta_o + self.theta_e);
        let (sin_theta_o, cos_theta_o) = self.theta_o.sin_cos();

        let a = 2.0 * f64::PI * (1.0 - cos_theta_o);
        let b = 2.0 * theta_w * sin_theta_o
            - (self.theta_o - 2.0 * theta_w).cos()
            - 2.0 * self.theta_o * sin_theta_o
            + cos_theta_o;
        a + f64::PI_OVER_2 * b
    }
}

/// The number of bins to use when deciding how to split the tree
const BIN_COUNT: usize = 12;
/// If the number of primitives hits this count, automatically create a leaf
const MIN_LIGHT_LEAF_COUNT: usize = 4;

pub struct LightBVH {
    lights: Vec<usize>, // An array of light indices
    nodes: Vec<Node>,
}

impl LightBVH {
    pub fn new(original_lights: &[&dyn Light]) -> Self {
        // First we go ahead and create a bunch of light info:
        let mut lights = Vec::with_capacity(original_lights.len());
        for (index, light) in original_lights.iter().enumerate() {
            lights.push(LightInfo {
                index,
                bound: light.get_bound(),
                centroid: light.get_centroid(),
            })
        }

        // Construct the nodes:
        let mut build_nodes = Vec::new();
        let mut ordered_lights: Vec::new();
        Self::rec_construct_bvh(&mut lights, &mut ordered_lights, &mut build_nodes);

        // Now go ahead and compact it linearly:
    }

    // Recursively constructs the bvh:
    fn rec_construct_bvh(
        lights: &mut [LightInfo],
        ordered_lights: &mut Vec<LightInfo>,
        build_nodes: &mut Vec<BuildNode>,
    ) {
        let global_bound = lights
            .iter()
            .fold(LightBound::new_initial(), |accum, light| {
                accum.combine(light.bound)
            });

        // Check the number of lights and see if we should make a leaf or not:
        if lights.len() < MIN_LIGHT_LEAF_COUNT {
            let light_index = ordered_lights.len();
            ordered_lights.extend(lights.iter());
            build_nodes.push(BuildNode::Leaf {
                bound: global_bound,
                light_index,
                num_lights: lights.len(),
            })
        }

        // Otherwise, we can try to split:
        match split_clusters(lights, global_bound) {
            Some((left, right)) => {
                // We recursively build the left and right one:
                Self::rec_construct_bvh(left, ordered_lights, build_nodes);
                Self::rec_construct_bvh(right, ordered_lights, build_nodes);
            }
            None => {
                // Don't bother splitting (not worth the cost), so go ahead:
                let light_index = ordered_lights.len();
                ordered_lights.extend(lights.iter());
                build_nodes.push(BuildNode::Leaf {
                    bound: global_bound,
                    light_index,
                    num_lights: lights.len(),
                })
            }
        }
    }
}

/// The final node used in the tree.
#[derive(Clone, Copy, Debug)]
struct Node {}

/// The Node used when constructing the tree.
#[derive(Clone, Copy, Debug)]
enum BuildNode {
    Internal {
        bound: LightBound,
        left: usize,
        right: usize,
    },
    Leaf {
        bound: LightBound,
        light_index: usize,
        num_lights: usize,
    },
}

/// Describes the bound over a bunch of lights:
#[derive(Clone, Copy, Debug)]
pub struct LightBound {
    pub bbox: BBox3<f64>,
    pub cone: Cone,
    pub power: f64,
}

impl LightBound {
    fn new_initial() -> Self {
        LightBound {
            bbox: BBox3::new_initial(),
            cone: Cone::new_initial(),
            power: 0.0,
        }
    }

    fn combine(self, bound: LightBound) -> Self {
        LightBound {
            bbox: self.bbox.combine_bnd(bound.bbox),
            cone: self.cone.combine(bound.cone),
            power: self.power + bound.power,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct LightInfo {
    index: usize,      // The index of the light
    bound: LightBound, // The bound over the lights
    centroid: Vec3<f64>,
}

// TODO: figure out naming convention and what to do about global_bin (which doesn't really make sense).
// Maybe call it a "cluster" and have LightInfo contain cluster information as well?

/// Attempts to split the cluster along a given axis. Returns a pair if a split was performed. If no
/// split was performed (because it wasn't worht it), then `None` is returned.
///
/// # Arguments
/// * `lights` - A collection of all of the light info we are currently working with
/// * `bound`  - The overall bound of all of the lights
/// * `cone`   - The overall cone of all of the lights
fn split_clusters(
    lights: &mut [LightInfo],
    global_bound: LightBound,
) -> Option<(&mut [LightInfo], &mut [LightInfo])> {
    // These values are used for the regularization factor:
    let bbox_diagonal = global_bound.bbox.diagonal();
    let bbox_max_length = bbox_diagonal[bbox_diagonal.max_dim()];

    let mut global_min_cost = f64::INFINITY;
    let mut global_min_bin = 0;
    let mut global_min_axis = 0;

    // Stores all of the potential splits across a number of different bins:
    let mut global_bins = [[LightBound::new_initial(); BIN_COUNT]; 3];

    // Look for the best split across all of the different axises:
    for (axis, bins) in global_bins.iter_mut().enumerate() {
        // Go through all the lights and place them into different sets of buckets:
        for l in lights.iter() {
            // Get the bucket index for the current primitive:
            let b = (BIN_COUNT as f64) * global_bound.bbox.offset(l.centroid)[axis];
            let b = if b >= (BIN_COUNT as f64) {
                BIN_COUNT - 1
            } else {
                b.floor() as usize
            };

            bins[b] = bins[b].combine(l.bound);
        }

        let mut min_cost = f64::INFINITY;
        let mut min_bin = 0;

        // Compute the regularization factor so that thin bounds aren't taken:
        let kr = bbox_max_length / bbox_diagonal[axis];

        for b in 0..(BIN_COUNT - 1) {
            // Combine everything up to bin b (inclusive):
            let left_bins = &bins[0..=b]
                .iter()
                .fold(LightBound::new_initial(), |accum, &bin| accum.combine(bin));

            // Combine everything after bin b:
            let right_bins = &bins[(b + 1)..BIN_COUNT]
                .iter()
                .fold(LightBound::new_initial(), |accum, &bin| accum.combine(bin));

            // The
            let left_cost = left_bins.power
                * left_bins.bbox.surface_area()
                * left_bins.cone.surface_area_orientation_heuristic();
            let right_cost = right_bins.power
                * right_bins.bbox.surface_area()
                * right_bins.cone.surface_area_orientation_heuristic();
            let cost = kr
                * ((left_cost + right_cost)
                    / (global_bound.bbox.surface_area()
                        * global_bound.cone.surface_area_orientation_heuristic()));

            if cost < min_cost {
                min_cost = cost;
                min_bin = b;
            }
        }

        if min_cost < global_min_cost {
            global_min_cost = min_cost;
            global_min_bin = min_bin;
            global_min_axis = axis;
        }
    }

    // Now check if we should perform a split or not (if it's not worth it, then we don't).
    if global_min_cost >= global_bound.power {
        return None;
    }

    // Now we go ahead and perform the partition:
    let (left_part, right_part) = partition::partition(lights, |l| {
        let b = (BIN_COUNT as f64) * global_bound.bbox.offset(l.centroid)[global_min_axis];
        let b = if b >= (BIN_COUNT as f64) {
            BIN_COUNT - 1
        } else {
            b.floor() as usize
        };
        b <= global_min_bin
    });

    Some((left_part, right_part))
}
