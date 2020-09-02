// // This code is based on the paper from:
// // Importance Sampling of Many Lights with Adaptive Tree Splitting by
// // Estevez and Kulla.

// use crate::light::Light;
// use partition;
// use pmath::bbox::BBox3;
// use pmath::matrix::Mat3x4;
// use pmath::numbers::Float;
// use pmath::vector::Vec3;

// /// A light cone represents the extent of a light
// #[derive(Copy, Clone, Debug)]
// pub struct Cone {
//     axis: Vec3<f64>,
//     theta_o: f64, // All angles are in radians
//     theta_e: f64,
// }

// impl Cone {
//     /// Construct an "initial" `LightCone`. This cone will equal whatever cone ii is combined with.
//     pub fn new_initial() -> Self {
//         Cone {
//             axis: Vec3::zero(),
//             theta_o: 0.0,
//             theta_e: 0.0,
//         }
//     }

//     /// Construct a new `LightCone`:
//     pub fn new(axis: Vec3<f64>, theta_o: f64, theta_e: f64) -> Self {
//         Cone {
//             axis: axis.normalize(),
//             theta_o: theta_o.max(0.0).min(f64::PI), // Make sure theta_o is in [0, PI]
//             theta_e: theta_e.max(0.0).min(f64::PI_OVER_2), // Make sure theta_e is in [0, PI/2]
//         }
//     }

//     /// Combines two `LightCone`s into one `LightCone` that encompasses everything.
//     fn combine(self, b: Cone) -> Self {
//         // Ensure that a.theta_o > b.theta_o
//         let (a, b) = if self.theta_o > b.theta_o {
//             (self, b)
//         } else {
//             (b, self)
//         };

//         let theta_d = a.axis.dot(b.axis).acos();
//         let theta_e = a.theta_e.max(b.theta_e);

//         if f64::PI.min(theta_d + b.theta_o) <= a.theta_o {
//             return Cone {
//                 axis: a.axis,
//                 theta_o: a.theta_o,
//                 theta_e,
//             };
//         }

//         let theta_o = (a.theta_o + theta_d + b.theta_o) * 0.5;
//         if f64::PI <= theta_o {
//             return Cone {
//                 axis: a.axis,
//                 theta_o: f64::PI,
//                 theta_e,
//             };
//         }

//         let theta_r = theta_o - a.theta_o;
//         let axis = {
//             // Create a rotation matrix around a.axis x b.axis:
//             let rot_mat = Mat3x4::new_rotate(theta_r.to_degrees(), a.axis.cross(b.axis));
//             rot_mat.mul_vec_zero(a.axis)
//         };

//         Cone {
//             axis,
//             theta_o,
//             theta_e,
//         }
//     }

//     fn surface_area_orientation_heuristic(self) -> f64 {
//         let theta_w = f64::PI.min(self.theta_o + self.theta_e);
//         let (sin_theta_o, cos_theta_o) = self.theta_o.sin_cos();

//         let a = 2.0 * f64::PI * (1.0 - cos_theta_o);
//         let b = 2.0 * theta_w * sin_theta_o
//             - (self.theta_o - 2.0 * theta_w).cos()
//             - 2.0 * self.theta_o * sin_theta_o
//             + cos_theta_o;
//         a + f64::PI_OVER_2 * b
//     }
// }

// /// The number of bins to use when deciding how to split the tree
// const BIN_COUNT: usize = 12;
// /// If the number of primitives hits this count, automatically create a leaf
// const MIN_LIGHT_LEAF_COUNT: usize = 4;

// /// This holds all of the shading information needed at a shading point
// /// to perform the necessary computations.
// #[derive(Clone, Copy, Debug)]
// pub struct ShadingInfo {
//     pub pos: Vec3<f64>, // In world space
//     pub nrm: Vec3<f64>, // In world space
// }

// pub struct LightBVH {
//     lights: Vec<LightInfo>, // An array of light indices
//     nodes: Vec<Node>,
// }

// impl LightBVH {
//     const MAX_LIGHT_PER_LEAF: usize = 16;

//     pub fn new(scene_lights: &[&dyn Light]) -> Self {
//         // First we go ahead and create a bunch of light info structures:
//         let lights: Vec<_> = scene_lights
//             .iter()
//             .enumerate()
//             .map(|(index, light)| LightInfo {
//                 index,
//                 bound: light.get_bound(),
//                 centroid: light.get_centroid(),
//             })
//             .collect();

//         // Then construct the bvh recursively:
//         let mut nodes = Vec::new();
//         let mut ordered_lights = Vec::new();
//         Self::rec_construct_bvh(&mut lights, &mut ordered_lights, &mut nodes);

//         nodes.shrink_to_fit();
//         ordered_lights.shrink_to_fit();

//         // Now go ahead and return them:
//         LightBVH {
//             lights: ordered_lights,
//             nodes,
//         }
//     }

//     // Given a shading point and a random value, returns the index of the light:
//     pub fn sample(&self, shading_info: ShadingInfo, u: f64) -> usize {
//         self.rec_sample(0, shading_info, u)
//     }

//     fn rec_sample(&self, curr_root: usize, shading_info: ShadingInfo, u: f64) -> usize {
//         let mut pdfs = [0.0; Self::MAX_LIGHT_PER_LEAF];
//         match self.nodes[curr_root] {
//             Node::Leaf {
//                 bound,
//                 light_index,
//                 num_lights,
//             } => {
//                 for (l, pdfs) in self.lights[light_index..(light_index + num_lights)]
//                     .iter()
//                     .zip(pdfs.iter())
//                 {
//                     *pdfs = Self::importance(l.bound, shading_info);
//                 }
//                 Self::sample_discrete_pdf(&pdfs, u)
//             }
//             Node::Internal { bound, left, right } => {
//                 let i_left = Self::importance(bound, shading_info);
//                 let i_right = Self::importance(bound, shading_info);
//                 if u < i_left / (i_left + i_right) {
//                     let u = u * (i_left + i_right) / i_left;
//                     self.rec_sample(left, shading_info, u)
//                 } else {
//                     let u = (u * (i_left + i_right) - i_left) / i_right;
//                     self.rec_sample(right, shading_info, u)
//                 }
//             }
//         }
//     }

//     // This isn't the most efficient implementation, but it's good enough for now...
//     fn importance(bound: LightBound, shading_info: ShadingInfo) -> f64 {
//         let bbox_center = bound.bbox.center();
//         let d = bbox_center - shading_info.pos;
//         let theta_i = {
//             let dot = d.normalize().dot(shading_info.nrm.normalize());
//             if dot < 0.0 {
//                 dot.acos()
//             } else {
//                 dot.acos()
//             }
//         };

//         let d2 = d.length2();
//         0.
//     }

//     /// Given a collection of pdfs and a random value, return the light index that
//     /// we had sampled here.
//     fn sample_discrete_pdf(pdfs: &[f64], u: f64) -> usize {
//         // Normalize the bloody pdf by summing over them:
//         let inv_total_pdfs = {
//             let total_pdfs: f64 = pdfs.iter().sum();
//             1.0 / total_pdfs
//         };
//         let mut curr_cdf = 0.0;
//         for (i, &pdf) in pdfs.iter().enumerate() {
//             curr_cdf += pdf * inv_total_pdfs;
//             if curr_cdf > u {
//                 return i;
//             }
//         }
//         0
//     }

//     /// Recursively constructs the bvh given a collection of lights.
//     fn rec_construct_bvh(
//         lights: &mut [LightInfo],
//         ordered_lights: &mut Vec<LightInfo>,
//         nodes: &mut Vec<Node>,
//     ) {
//         let global_bound = lights
//             .iter()
//             .fold(LightBound::new_initial(), |accum, light| {
//                 accum.combine(light.bound)
//             });

//         // Check the number of lights and see if we should make a leaf or not:
//         if lights.len() < MIN_LIGHT_LEAF_COUNT {
//             let light_index = ordered_lights.len();
//             ordered_lights.extend(lights.iter());
//             nodes.push(Node::Leaf {
//                 bound: global_bound,
//                 light_index,
//                 num_lights: lights.len(),
//             })
//         }

//         // Otherwise, we can try to split:
//         match Self::split_clusters(lights, global_bound) {
//             Some((left, right)) => {
//                 // We recursively build the left and right one:
//                 Self::rec_construct_bvh(left, ordered_lights, nodes);
//                 Self::rec_construct_bvh(right, ordered_lights, nodes);
//             }
//             None => {
//                 // Don't bother splitting (not worth the cost), so go ahead:
//                 let light_index = ordered_lights.len();
//                 ordered_lights.extend(lights.iter());
//                 nodes.push(Node::Leaf {
//                     bound: global_bound,
//                     light_index,
//                     num_lights: lights.len(),
//                 })
//             }
//         }
//     }

//     /// Attempts to split the cluster along a given axis. Returns a pair if a split was performed. If no
//     /// split was performed (because it wasn't worht it), then `None` is returned.
//     ///
//     /// # Arguments
//     /// * `lights` - A collection of all of the light info we are currently working with
//     /// * `bound`  - The overall bound of all of the lights
//     /// * `cone`   - The overall cone of all of the lights
//     fn split_clusters(
//         lights: &mut [LightInfo],
//         global_bound: LightBound,
//     ) -> Option<(&mut [LightInfo], &mut [LightInfo])> {
//         // These values are used for the regularization factor:
//         let bbox_diagonal = global_bound.bbox.diagonal();
//         let bbox_max_length = bbox_diagonal[bbox_diagonal.max_dim()];

//         let mut global_min_cost = f64::INFINITY;
//         let mut global_min_bin = 0;
//         let mut global_min_axis = 0;

//         // Stores all of the potential splits across a number of different bins:
//         let mut global_bins = [[LightBound::new_initial(); BIN_COUNT]; 3];

//         // Look for the best split across all of the different axises:
//         for (axis, bins) in global_bins.iter_mut().enumerate() {
//             // Go through all the lights and place them into different sets of buckets:
//             for l in lights.iter() {
//                 // Get the bucket index for the current primitive:
//                 let b = (BIN_COUNT as f64) * global_bound.bbox.offset(l.centroid)[axis];
//                 let b = if b >= (BIN_COUNT as f64) {
//                     BIN_COUNT - 1
//                 } else {
//                     b.floor() as usize
//                 };

//                 bins[b] = bins[b].combine(l.bound);
//             }

//             let mut min_cost = f64::INFINITY;
//             let mut min_bin = 0;

//             // Compute the regularization factor so that thin bounds aren't taken:
//             let kr = bbox_max_length / bbox_diagonal[axis];

//             for b in 0..(BIN_COUNT - 1) {
//                 // Combine everything up to bin b (inclusive):
//                 let left_bins = &bins[0..=b]
//                     .iter()
//                     .fold(LightBound::new_initial(), |accum, &bin| accum.combine(bin));

//                 // Combine everything after bin b:
//                 let right_bins = &bins[(b + 1)..BIN_COUNT]
//                     .iter()
//                     .fold(LightBound::new_initial(), |accum, &bin| accum.combine(bin));

//                 // The
//                 let left_cost = left_bins.power
//                     * left_bins.bbox.surface_area()
//                     * left_bins.cone.surface_area_orientation_heuristic();
//                 let right_cost = right_bins.power
//                     * right_bins.bbox.surface_area()
//                     * right_bins.cone.surface_area_orientation_heuristic();
//                 let cost = kr
//                     * ((left_cost + right_cost)
//                         / (global_bound.bbox.surface_area()
//                             * global_bound.cone.surface_area_orientation_heuristic()));

//                 if cost < min_cost {
//                     min_cost = cost;
//                     min_bin = b;
//                 }
//             }

//             if min_cost < global_min_cost {
//                 global_min_cost = min_cost;
//                 global_min_bin = min_bin;
//                 global_min_axis = axis;
//             }
//         }

//         // Now check if we should perform a split or not (if it's not worth it, then we don't).
//         // If we are over MAX_LIGHT_PER_LEAF, then we have to keep going regardless:
//         if (lights.len() <= Self::MAX_LIGHT_PER_LEAF) && (global_min_cost >= global_bound.power) {
//             return None;
//         }

//         // Now we go ahead and perform the partition:
//         let (left_part, right_part) = partition::partition(lights, |l| {
//             let b = (BIN_COUNT as f64) * global_bound.bbox.offset(l.centroid)[global_min_axis];
//             let b = if b >= (BIN_COUNT as f64) {
//                 BIN_COUNT - 1
//             } else {
//                 b.floor() as usize
//             };
//             b <= global_min_bin
//         });

//         Some((left_part, right_part))
//     }
// }

// /// The Node used when constructing the tree.
// #[derive(Clone, Copy, Debug)]
// enum Node {
//     Internal {
//         bound: LightBound,
//         left: usize,
//         right: usize,
//     },
//     Leaf {
//         bound: LightBound,
//         light_index: usize,
//         num_lights: usize,
//     },
// }

// /// Describes the bound over a bunch of lights:
// #[derive(Clone, Copy, Debug)]
// pub struct LightBound {
//     pub bbox: BBox3<f64>,
//     pub cone: Cone,
//     pub power: f64,
// }

// impl LightBound {
//     fn new_initial() -> Self {
//         LightBound {
//             bbox: BBox3::new_initial(),
//             cone: Cone::new_initial(),
//             power: 0.0,
//         }
//     }

//     fn combine(self, bound: LightBound) -> Self {
//         LightBound {
//             bbox: self.bbox.combine_bnd(bound.bbox),
//             cone: self.cone.combine(bound.cone),
//             power: self.power + bound.power,
//         }
//     }
// }

// #[derive(Clone, Copy, Debug)]
// struct LightInfo {
//     index: usize,      // The index of the light
//     bound: LightBound, // The bound over the lights
//     centroid: Vec3<f64>,
// }
