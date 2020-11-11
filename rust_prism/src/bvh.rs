// This code is based on the paper from:
// Importance Sampling of Many Lights with Adaptive Tree Splitting by
// Estevez and Kulla.

use crate::interaction::Interaction;
use arrayvec::ArrayVec;
use partition;
use pmath::bbox::BBox3;
use pmath::ray::Ray;
use pmath::vector::Vec3;

/// A trait for a BVH object. For certain use cases (like when constructing
/// a BVH for a triangular mesh), it may be more efficient to store the primitive
/// directly with this trait. For other cases (with a number of different BVH objects),
/// a BVHObject can simply be a reference.
pub trait BVHObject: Clone {
    type UserData;

    fn get_bbox(&self, user_data: &Self::UserData) -> BBox3<f64>;

    fn intersect_test(&self, ray: Ray<f64>, user_data: &Self::UserData) -> bool;
    fn intersect(&self, ray: Ray<f64>, user_data: &Self::UserData) -> Option<Interaction>;
}

pub struct BVH<Object: BVHObject> {
    objects: Vec<Object>,
    nodes: Vec<Node>,
    bbox: BBox3<f64>,
}

impl<Object: BVHObject> BVH<Object> {
    const SAH_BIN_COUNT: usize = 12;

    /// Given a collection of BVH objects, constructs a BVH.
    pub fn new(objects: &[Object], max_per_leaf: usize, user_data: &Object::UserData) -> Self {
        // First we go ahead and create a bunch of light info structures:
        let mut object_infos: Vec<_> = objects
            .iter()
            .enumerate()
            .map(|(index, object)| {
                let bbox = object.get_bbox(user_data);
                ObjectInfo {
                    index,
                    bbox,
                    centroid: bbox.centroid(),
                }
            })
            .collect();

        let global_bbox = object_infos
            .iter()
            .fold(BBox3::new_initial(), |accum, object_info| {
                accum.combine_bnd(object_info.bbox)
            });

        // Then construct the bvh recursively:
        let mut nodes = Vec::new();
        let mut ordered_objects = Vec::with_capacity(objects.len());
        Self::rec_construct_bvh(
            &mut object_infos,
            &mut ordered_objects,
            objects,
            &mut nodes,
            max_per_leaf,
            global_bbox,
        );

        nodes.shrink_to_fit();
        ordered_objects.shrink_to_fit();

        // Now go ahead and return them:
        BVH {
            objects: ordered_objects,
            nodes,
            bbox: global_bbox,
        }
    }

    pub fn get_bbox(&self) -> BBox3<f64> {
        self.bbox
    }

    pub fn get_objects(&self) -> &[Object] {
        &self.objects[..]
    }

    /// Given a `Ray`, performs an intersection test, simply returning true if the ray intersects any object in
    /// the BVH and false otherwise.
    pub fn intersect_test(&self, ray: Ray<f64>, user_data: &Object::UserData) -> bool {
        // We do this because t_far may get updated:
        let inv_dir = ray.dir.inv_scale(1.0);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut stack = ArrayVec::<[_; 64]>::new();
        stack.push(0); // first index to visit

        loop {
            // Get the next node to visit. If no nodes are left, we are done:
            let node_index = match stack.pop() {
                Some(node_index) => node_index,
                None => return false,
            };

            // Check if we can intersect the point:
            let node = self.nodes[node_index];
            if node.bbox.intersect_test(ray, inv_dir, is_dir_neg) {
                match node.node_type {
                    NodeType::Leaf { index, count } => {
                        // Because we update the t_far variable, new hit is a closer hit:
                        for object in &self.objects[index..(index + count)] {
                            if object.intersect_test(ray, user_data) {
                                return true;
                            }
                        }
                    }
                    NodeType::Internal {
                        axis,
                        first,
                        second,
                    } => {
                        // We want to first intersect the child that is closest to the ray (to "prune" t_far)
                        // as much as possible first.
                        if is_dir_neg[axis] {
                            stack.push(first);
                            stack.push(second); // we are under the axis and "second" should be first
                        } else {
                            stack.push(second);
                            stack.push(first); // otherwise, "first" should be first
                        }
                    }
                }
            }
        }
    }

    /// Given a `Ray`, performs an intersection, returning a `GeomSurface` of the point of intersection.
    pub fn intersect(&self, ray: Ray<f64>, user_data: &Object::UserData) -> Option<Interaction> {
        // We do this because t_far may get updated:
        let inv_dir = ray.dir.inv_scale(1.0);
        let is_dir_neg = ray.dir.comp_wise_is_neg();
        let mut ray = ray;

        let mut stack = ArrayVec::<[_; 64]>::new();
        stack.push(0); // first index to visit

        let mut hit = None;

        loop {
            // Get the next node to visit:
            let node_index = match stack.pop() {
                Some(node_index) => node_index,
                None => return hit,
            };

            // Check if we can intersect the point:
            let node = self.nodes[node_index];
            if node.bbox.intersect_test(ray, inv_dir, is_dir_neg) {
                match node.node_type {
                    NodeType::Leaf { index, count } => {
                        // Because we update the extent, every new hit is a closer hit:
                        for object in &self.objects[index..(index + count)] {
                            if let Some(geom_surface) = object.intersect(ray, user_data) {
                                ray.t_far = geom_surface.t;
                                hit = Some(geom_surface);
                            }
                        }
                    }
                    NodeType::Internal {
                        axis,
                        first,
                        second,
                    } => {
                        // We want to first intersect the child that is closest to the ray (to "prune" t_far)
                        // as much as possible first.
                        if is_dir_neg[axis] {
                            stack.push(first);
                            stack.push(second); // we are under the axis and "second" should be first
                        } else {
                            stack.push(second);
                            stack.push(first); // otherwise, "first" should be first
                        }
                    }
                }
            }
        }
    }

    /// Recursively constructs the scene. Returns the index of the node that is constructed by the function call.
    ///
    /// # Arguments
    /// * `object_infos` - A collection of information about the objects we are trying to split. This is mutable as it
    ///                    gets partitioned as we continue the process.
    /// * `ordered_objects` - The final order of the objects so that the nodes can index them.
    /// * `objects` - The original objects that ObjectInfo is indexing.
    fn rec_construct_bvh(
        object_infos: &mut [ObjectInfo],
        ordered_objects: &mut Vec<Object>,
        objects: &[Object],
        nodes: &mut Vec<Node>,
        max_per_leaf: usize,
        global_bbox: BBox3<f64>,
    ) -> usize {
        // Function that creates a leaf node:
        let create_leaf = || {
            let index = ordered_objects.len();
            ordered_objects.extend(
                object_infos
                    .iter()
                    .map(|object_info| objects[object_info.index].clone()),
            );
            nodes.push(Node {
                bbox: global_bbox,
                node_type: NodeType::Leaf {
                    index,
                    count: object_infos.len(),
                },
            })
        };

        // Check the number of lights and see if we should make a leaf or not:
        if object_infos.len() < max_per_leaf {
            create_leaf();
            return nodes.len() - 1;
        }

        // Otherwise, we try performing a split:
        match Self::split_clusters(object_infos, global_bbox) {
            Some((first_object_infos, second_object_infos, axis)) => {
                // We push this here so that we can easily calculate the overal global bound and store it at the
                // "top level".

                let first_global_bbox = first_object_infos
                    .iter()
                    .fold(BBox3::new_initial(), |accum, object_info| {
                        accum.combine_bnd(object_info.bbox)
                    });

                let second_global_bbox = second_object_infos
                    .iter()
                    .fold(BBox3::new_initial(), |accum, object_info| {
                        accum.combine_bnd(object_info.bbox)
                    });

                // We recursively build the left and right one:
                let first = Self::rec_construct_bvh(
                    first_object_infos,
                    ordered_objects,
                    objects,
                    nodes,
                    max_per_leaf,
                    first_global_bbox,
                );
                let second = Self::rec_construct_bvh(
                    second_object_infos,
                    ordered_objects,
                    objects,
                    nodes,
                    max_per_leaf,
                    second_global_bbox,
                );

                // Construct an internal node and add it to the node vector:
                nodes.push(Node {
                    bbox: global_bbox,
                    node_type: NodeType::Internal {
                        axis,
                        first,
                        second,
                    },
                })
            }
            None => create_leaf(),
        }

        nodes.len() - 1
    }

    /// Attempts to split the cluster along a given axis. Returns a pair of slices and the axis where the split occured
    /// if a split was performed. If no split was performed (because it wasn't worth it), then `None` is returned.
    ///
    /// # Arguments
    /// * `object_infos` - A collection of information about the objects we are trying to split.
    /// * `global_bound` - The overall bound of all of the objects that we are trying to split.
    fn split_clusters(
        object_infos: &mut [ObjectInfo],
        global_bbox: BBox3<f64>,
    ) -> Option<(&mut [ObjectInfo], &mut [ObjectInfo], usize)> {
        // These values are used for the regularization factor:
        let bbox_diagonal = global_bbox.diagonal();
        let bbox_max_length = bbox_diagonal[bbox_diagonal.max_dim()];

        let mut global_min_cost = f64::INFINITY;
        let mut global_min_bin = 0;
        let mut global_min_axis = 0;

        // Stores all of the potential splits across the different axises (there are 3 of them):
        let mut global_bins = [[SAHBin::new(); Self::SAH_BIN_COUNT]; 3];

        // Look for the best split across all of the different axises:
        for (axis, bins) in global_bins.iter_mut().enumerate() {
            // Go through all the objects and place them into different sets of buckets:
            for object_info in object_infos.iter() {
                // Get the bucket index for the current primitive:
                let b =
                    (Self::SAH_BIN_COUNT as f64) * global_bbox.offset(object_info.centroid)[axis];
                let b = if b >= (Self::SAH_BIN_COUNT as f64) {
                    Self::SAH_BIN_COUNT - 1
                } else {
                    b as usize
                };

                bins[b] = bins[b].add_object(object_info.bbox);
            }

            let mut min_cost = f64::INFINITY;
            let mut min_bin = 0;

            // Find the bin that would lead to the best heuristic for this axis:
            for b in 0..(Self::SAH_BIN_COUNT - 1) {
                // Combine everything up to bin b (inclusive):
                let left_bins = bins[0..=b]
                    .iter()
                    .fold(SAHBin::new(), |accum, &bin| accum.combine(bin));

                // Combine everything after bin b:
                let right_bins = bins[(b + 1)..Self::SAH_BIN_COUNT]
                    .iter()
                    .fold(SAHBin::new(), |accum, &bin| accum.combine(bin));

                let cost = 1.0
                    + ((left_bins.count as f64) * left_bins.bbox.surface_area()
                        + (right_bins.count as f64) * right_bins.bbox.surface_area())
                        / global_bbox.surface_area();

                if cost < min_cost {
                    min_cost = cost;
                    min_bin = b;
                }
            }

            // Check how this cost compares to the other axis.
            if min_cost < global_min_cost {
                global_min_cost = min_cost;
                global_min_bin = min_bin;
                global_min_axis = axis;
            }
        }

        // Now check if we should perform a split or not. If we assume that every leaf has a cost of 1, then we have
        // that it's not worth it if the cost is greater or equal to the number of objects:
        if global_min_cost >= (object_infos.len() as f64) {
            return None;
        }

        // Now we go ahead and perform the partition:
        let (first_part, second_part) = partition::partition(object_infos, |object_info| {
            let b = (Self::SAH_BIN_COUNT as f64)
                * global_bbox.offset(object_info.centroid)[global_min_axis];
            let b = if b >= (Self::SAH_BIN_COUNT as f64) {
                Self::SAH_BIN_COUNT - 1
            } else {
                b as usize
            };
            b <= global_min_bin
        });

        Some((first_part, second_part, global_min_axis))
    }
}

#[derive(Clone, Copy, Debug)]
struct Node {
    bbox: BBox3<f64>,
    node_type: NodeType,
}

/// A node for BVH stuff. Note that this probably isn't very
#[derive(Clone, Copy, Debug)]
enum NodeType {
    Internal {
        axis: usize, // The axis where we split
        first: usize,
        second: usize,
    },
    Leaf {
        index: usize, // This specifies the range over the objects that belong to it
        count: usize,
    },
}

struct ObjectInfo {
    index: usize,     // The index of the light
    bbox: BBox3<f64>, // The bound over the lights (TODO: make bounds generic?)
    centroid: Vec3<f64>,
}

/// The bins that we use to traverse the BVH. Note that
/// it's aligned to a cache line (64 bytes).
#[derive(Clone, Copy, Debug)]
struct SAHBin {
    bbox: BBox3<f64>,
    count: u32,
}

impl SAHBin {
    fn new() -> Self {
        SAHBin {
            bbox: BBox3::new_initial(),
            count: 0,
        }
    }

    /// Combines two different bins.
    fn combine(self, o: SAHBin) -> Self {
        SAHBin {
            bbox: self.bbox.combine_bnd(o.bbox),
            count: self.count + o.count,
        }
    }

    /// Updates the bin with an object's bbox.
    fn add_object(self, bbox: BBox3<f64>) -> Self {
        SAHBin {
            bbox: self.bbox.combine_bnd(bbox),
            count: self.count + 1,
        }
    }
}
