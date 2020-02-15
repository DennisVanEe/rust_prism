use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::Vec3;

use arrayvec::ArrayVec;
use bumpalo::Bump;
use order_stat::kth_by;
use partition::partition;

// A trait that must be implemented for any type type that wishes to be a part of 
pub trait BVHObject {
    // Any additional information for calculating the intersection (like if it's a triangular mesh)
    type IntParam;
    // Any additional information for calculating the bounds or centroids:
    type DataParam;
    // What is returned when intersecting the object:
    type IntResult;

    // The intersection algorithms need to support potentially moving objects:
    fn intersect_test(
        &self,
        ray: Ray<f64>,
        max_t: f64,
        curr_time: f64,
        int_info: &Self::IntParam,
    ) -> bool;

    fn intersect(
        &self,
        ray: Ray<f64>,
        max_t: f64,
        curr_time: f64,
        int_info: &Self::IntParam,
    ) -> Option<IntResult>;

    fn get_centroid(&self, data: &Self::DataParam) -> Vec3<f64>;
    fn get_bound(&self, data: &Self::DataParam) -> BBox3<f64>;
}

// How many buckets we use for the SAH splitting algorithm:
const BUCKET_COUNT: usize = 12;

pub struct BVH<O: BVHObject> {
    objects: Vec<O>,
    linear_nodes: Vec<LinearNode>,
    bound: BBox3<f64>,
}

impl<O: BVHObject> BVH<O> {
    // Constructs a BVH given a mesh and the max number of triangles per leaf node.
    // The BVH will become the owner of the mesh when doing this.
    pub fn new(objects: Vec<O>, max_obj_per_node: usize, data: &O::DataParam) -> Self {
        // First we record any object information we may need:
        let mut object_infos = Vec::with_capacity(objects.len());
        for (i, obj) in objects.iter().enumerate() {
            object_infos.push(ObjectInfo {
                obj_index: i as u32,
                centroid: obj.get_centroid(data),
                bound: obj.get_bound(data),
            });
        }

        // Now we can go ahead and construct the tree:
        Self::construct_tree(objects, object_infos, max_obj_per_node)
    }

    pub fn get_bound(&self) -> BBox3<f64> {
        self.bound
    }

    // Sometimes one needs to access this information (like Mesh):
    pub fn get_objects(&self) -> &Vec<O> {
        &self.objects
    }

    pub fn intersect(
        &self,
        ray: Ray<f64>,
        mut max_t: f64,
        curr_time: f64,
        int_info: &O::IntParam,
    ) -> Option<(O::IntResult, &O)> {
        // This function has to be very efficient, so I'll be using a lot of unsafe code
        // here (but everything I'm doing should still be defined behavior).
        let inv_dir = ray.dir.inv_scale(1.);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut node_stack = ArrayVec::<[_; 64]>::new();
        let mut curr_node_index = 0usize;

        // This is the final result:
        let mut result = None;

        loop {
            let curr_node = *unsafe { self.linear_nodes.get_unchecked(curr_node_index) };
            if curr_node
                .bound
                .intersect_test(ray, max_t, inv_dir, is_dir_neg)
            {
                match curr_node.kind {
                    LinearNodeKind::Leaf {
                        obj_start_index,
                        obj_end_index,
                    } => {
                        let obj_start = obj_start_index as usize;
                        let obj_end = obj_end_index as usize;
                        unsafe {
                            for obj in self.objects.get_unchecked(obj_start..obj_end).iter() {
                                if let Some(intersection) =
                                    obj.intersect(ray, max_t, curr_time, int_info)
                                {
                                    // Update the max time for more efficient culling:
                                    max_t = intersection.t;
                                    // Can't return immediately, have to make sure this is the closest intersection
                                    result = Some((intersection, obj));
                                }
                            }
                        }

                        // Pop the stack (if it's empty, we are done):
                        curr_node_index = match node_stack.pop() {
                            Some(i) => i,
                            _ => break,
                        };
                    }
                    LinearNodeKind::Interior {
                        right_child_index,
                        split_axis,
                    } => {
                        // Check which child it's most likely to be:
                        if is_dir_neg[split_axis as usize] {
                            // Push the first child onto the stack to perform later:
                            unsafe {
                                node_stack.push_unchecked(curr_node_index + 1);
                            }
                            curr_node_index = right_child_index as usize;
                        } else {
                            // Push the second child onto the stack to perform later:
                            unsafe {
                                node_stack.push_unchecked(right_child_index as usize);
                            }
                            curr_node_index += 1; // the first child
                        }
                    }
                }
            // If we don't hit it, then we try another item from the stack:
            } else {
                curr_node_index = match node_stack.pop() {
                    Some(i) => i,
                    _ => break,
                };
            }
        }

        result
    }

    pub fn intersect_test(
        &self,
        ray: Ray<f64>,
        max_t: f64,
        curr_time: f64,
        int_info: &O::IntParam,
    ) -> bool {
        // This function has to be very efficient, so I'll be using a lot of unsafe code
        // here (but everything I'm doing should still be defined behavior).

        let inv_dir = ray.dir.inv_scale(1.);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut node_stack = ArrayVec::<[_; 64]>::new();
        let mut curr_node_index = 0usize;

        loop {
            let curr_node = *unsafe { self.linear_nodes.get_unchecked(curr_node_index) };
            if curr_node
                .bound
                .intersect_test(ray, max_t, inv_dir, is_dir_neg)
            {
                match curr_node.kind {
                    LinearNodeKind::Leaf {
                        obj_start_index,
                        obj_end_index,
                    } => {
                        let obj_start = obj_start_index as usize;
                        let obj_end = obj_end_index as usize;
                        unsafe {
                            for obj in self.objects.get_unchecked(obj_start..obj_end).iter() {
                                if obj.intersect_test(ray, max_t, curr_time, int_info) {
                                    return true;
                                }
                            }
                        }

                        // Pop the stack (if it's empty, we are done):
                        curr_node_index = match node_stack.pop() {
                            Some(i) => i,
                            _ => return false,
                        };
                    }
                    LinearNodeKind::Interior {
                        right_child_index,
                        split_axis,
                    } => {
                        // Check which child it's most likely to be:
                        if is_dir_neg[split_axis as usize] {
                            // Push the first child onto the stack to perform later:
                            unsafe {
                                node_stack.push_unchecked(curr_node_index + 1);
                            }
                            curr_node_index = right_child_index as usize;
                        } else {
                            // Push the second child onto the stack to perform later:
                            unsafe {
                                node_stack.push_unchecked(right_child_index as usize);
                            }
                            curr_node_index += 1; // the first child
                        }
                    }
                }
            // If we don't hit it, then we try another item from the stack:
            } else {
                curr_node_index = match node_stack.pop() {
                    Some(i) => i,
                    _ => return false,
                };
            }
        }
    }

    // Given a mesh, triangle info (as passed by new), and the number of triangles per node,
    // construct a tree:
    fn construct_tree(
        mut objects: Vec<O>,
        mut object_infos: Vec<ObjectInfo>,
        max_obj_per_node: usize,
    ) -> Self {
        // Used to allocate the nodes:
        let mut allocator = Bump::new();
        // The new triangles that will replace the ones in Mesh (they will be ordered
        // in the correct manner):
        let mut new_objects = Vec::with_capacity(objects.len());

        // Construct the regular tree first (that isn't flat):
        let (root_node, bound, alloc_count) = Self::recursive_construct_tree(
            max_obj_per_node,
            &objects,
            &mut object_infos,
            &mut new_objects,
            &mut allocator,
            0,
        );

        // Now we flatten the nodes for better memory and performance later down the line:
        let linear_nodes = Self::flatten_tree(alloc_count, root_node);

        BVH {
            objects: new_objects,
            linear_nodes,
            bound,
        }
    }

    // Recursively constructs the tree.
    // Returns a reference to the root node of the tree and the bound of the entire tree that was created:
    fn recursive_construct_tree<'a>(
        max_obj_per_node: usize,         // The maximum number of triangles per node.
        objects: &Vec<O>,                // The mesh we are currently constructing a BVH for.
        object_infos: &mut [ObjectInfo], // The current slice of triangles we are working on.
        new_objects: &mut Vec<O>,        // The correct order for the new triangles we care about.
        allocator: &'a mut Bump, // Allocator used to allocate the nodes. The lifetime of the nodes is that of the allocator
        alloc_count: usize,      // This is used so we can efficiently allocate linear nodes.
    ) -> (&'a TreeNode<'a>, BBox3<f64>, usize) {
        // A bound over all of the triangles we are currently working with:
        let all_bound = object_infos
            .iter()
            .fold(BBox3::new(), |all_bound, obj_info| {
                all_bound.combine_bnd(obj_info.bound)
            });

        // If we only have one triangle, make a leaf:
        if object_infos.len() == 1 {
            new_objects.push(objects[object_infos[0].obj_index as usize]);
            return (
                allocator.alloc(TreeNode::Leaf {
                    bound: all_bound,
                    obj_index: (new_objects.len() - 1) as u32,
                    num_obj: 1,
                }),
                all_bound,
                alloc_count + 1,
            );
        }

        // Otherwise, we want to split the tree into smaller parts:

        // The bound covering all of the centroids (used for SAH BVH construction):
        let centroid_bound = object_infos
            .iter()
            .fold(BBox3::new(), |centroid_bound, obj_info| {
                centroid_bound.combine_pnt(obj_info.centroid)
            });

        // Now we want to split based on the largest dimension:
        let max_dim = centroid_bound.max_dim();

        // Check if the volume has volume 0, if so, then create a leaf node:
        if centroid_bound.pmax[max_dim] == centroid_bound.pmin[max_dim] {
            // Need to keep track of where we will be putting these triangles.
            let curr_obj_index = new_objects.len() as u32;
            for obj_info in object_infos.iter() {
                new_objects.push(objects[obj_info.obj_index as usize]);
            }
            // Allocate the a new leaf node and push it:
            return (
                allocator.alloc(TreeNode::Leaf {
                    bound: all_bound,
                    obj_index: curr_obj_index,
                    num_obj: object_infos.len() as u32,
                }),
                all_bound,
                alloc_count + 1,
            );
        }

        // Figure out how to split the elements:
        // If we have less than 4 triangles, just split it evenly:
        let (object_infos_left, object_infos_right) = if object_infos.len() <= 4 {
            // kth_by is essentially nth_element from C++.
            // Here, we reorder the triangles based on the value of the centroid
            // in the maximum dimension (dim).
            let mid = object_infos.len() / 2;
            kth_by(object_infos, mid, |obj_info0, obj_info1| {
                obj_info0.centroid[max_dim]
                    .partial_cmp(&obj_info1.centroid[max_dim])
                    .unwrap()
            });
            // Split the array:
            object_infos.split_at_mut(mid)
        } else {
            // Otherwise, we perform this split based on surface-area heuristics:
            let mut buckets = [Bucket {
                count: 0,
                bound: BBox3::new(),
            }; BUCKET_COUNT];

            for obj_info in object_infos.iter() {
                // Get an index into where we are among the buckets:
                let bucket_ratio = centroid_bound.offset(obj_info.centroid)[max_dim];
                let bucket_index = if bucket_ratio == 1. {
                    BUCKET_COUNT - 1
                } else {
                    ((BUCKET_COUNT as f64) * bucket_ratio) as usize
                };

                let curr_bucket = &mut buckets[bucket_index];
                curr_bucket.count += 1;
                curr_bucket.bound = curr_bucket.bound.combine_bnd(obj_info.bound);
            }

            // Iterate over everything backwards, but ignore the first element to get the right
            // surface area values:
            let mut right_sa = [0f64; BUCKET_COUNT - 1];
            let (_, right_count) = buckets[1..].iter().enumerate().rev().fold(
                (BBox3::new(), 0u32),
                |(right_bound, right_count), (i, bucket)| {
                    // Have to do this because enumerate starts at 0, always, not the index of the slice:
                    let right_bound = right_bound.combine_bnd(bucket.bound);
                    right_sa[i] = right_bound.surface_area();
                    (right_bound, right_count + bucket.count)
                },
            );

            // Now we can compute the values going forward to fill in the buckets.
            // We also must modify the right count as we decrement it over time:
            let mut costs = [0f64; BUCKET_COUNT - 1];
            let total_sa = all_bound.surface_area();
            buckets[..(BUCKET_COUNT - 1)].iter().enumerate().fold(
                (BBox3::new(), 0u32, right_count),
                |(left_bound, left_count, right_count), (i, bucket)| {
                    let left_bound = left_bound.combine_bnd(bucket.bound);
                    let left_count = left_count + bucket.count;
                    // Calculate the heuristic here:
                    costs[i] = 0.125
                        * ((left_count as f64) * left_bound.surface_area()
                            + (right_count as f64) * right_sa[i])
                        / total_sa;
                    (left_bound, left_count, right_count - buckets[i + 1].count)
                },
            );

            let (min_cost_index, &min_cost) = costs
                .iter() // returns a reference to the elements (so a &x essentially).
                .enumerate() // returns (i, &x), and max_by's lambda takes a reference. But coercion helps here:
                .min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
                .unwrap();

            // If this happens, then we should split more and continue our operations:
            if object_infos.len() > max_obj_per_node || min_cost < (object_infos.len() as f64) {
                // Split (partition) based on bucket with min cost:
                partition(object_infos, |obj_info| {
                    let bucket_ratio = centroid_bound.offset(obj_info.centroid)[max_dim];
                    let bucket_index = if bucket_ratio == 1. {
                        BUCKET_COUNT - 1
                    } else {
                        ((BUCKET_COUNT as f64) * bucket_ratio) as usize
                    };
                    bucket_index <= min_cost_index
                })
            } else {
                // Otherwise, it isn't worth it so continue the splitting process, so we
                // create a leaf here:
                let curr_obj_index = new_objects.len() as u32;
                for obj_info in object_infos.iter() {
                    new_objects.push(objects[obj_info.obj_index as usize]);
                }
                return (
                    allocator.alloc(TreeNode::Leaf {
                        bound: all_bound,
                        obj_index: curr_obj_index,
                        num_obj: object_infos.len() as u32,
                    }),
                    all_bound,
                    alloc_count + 1,
                );
            }
        };

        // Build the left and right nodes now:
        let (left_node, _, alloc_count) = Self::recursive_construct_tree(
            max_obj_per_node,
            objects,
            object_infos_left,
            new_objects,
            allocator,
            alloc_count,
        );
        let (right_node, _, alloc_count) = Self::recursive_construct_tree(
            max_obj_per_node,
            objects,
            object_infos_right,
            new_objects,
            allocator,
            alloc_count,
        );

        // Create the interior node and push it:
        (
            allocator.alloc(TreeNode::Interior {
                bound: all_bound,
                children: (left_node, right_node),
                split_axis: max_dim as u8,
            }),
            all_bound,
            alloc_count + 1,
        )
    }

    // Need to specify the tree node and the total number of nodes.
    // Will return the linear nodes as a vector.
    fn flatten_tree(num_nodes: usize, root_node: &TreeNode) -> Vec<LinearNode> {
        // This will generate the linear nodes we care about:
        fn generate_linear_nodes(
            linear_nodes: &mut Vec<LinearNode>,
            curr_node: &TreeNode,
        ) -> usize {
            match *curr_node {
                TreeNode::Leaf {
                    bound,
                    obj_index,
                    num_obj,
                } => {
                    linear_nodes.push(LinearNode {
                        bound,
                        kind: LinearNodeKind::Leaf {
                            obj_start_index: obj_index,
                            obj_end_index: obj_index + num_obj,
                        },
                    });
                    linear_nodes.len() - 1
                }
                TreeNode::Interior {
                    bound,
                    children: (left, right),
                    split_axis,
                } => {
                    let curr_pos = linear_nodes.len();
                    // Temporarily "push" a value:
                    unsafe { linear_nodes.set_len(curr_pos + 1) };
                    generate_linear_nodes(linear_nodes, left);
                    let right_child_index = generate_linear_nodes(linear_nodes, right) as u32;
                    *unsafe { linear_nodes.get_unchecked_mut(curr_pos) } = LinearNode {
                        bound,
                        kind: LinearNodeKind::Interior {
                            right_child_index,
                            split_axis,
                        },
                    };
                    curr_pos
                }
            }
        }

        // First create a vector with the correct number of nodes:
        let mut linear_nodes = Vec::with_capacity(num_nodes);
        let cnt = generate_linear_nodes(&mut linear_nodes, root_node);
        linear_nodes
    }
}

// This is the bucket used for SAH splitting:
#[derive(Clone, Copy)]
struct Bucket {
    // Number of items in the current bucket:
    pub count: u32,
    // Bound for the current bucket:
    pub bound: BBox3<f64>,
}

// Structure used to construct the BVH:
#[derive(Clone, Copy)]
struct ObjectInfo {
    pub obj_index: u32,
    pub centroid: Vec3<f64>,
    pub bound: BBox3<f64>,
}

// This is the internal representation we have when initially building the tree.
// We later "flatten" the tree for efficient traversal.
#[derive(Clone, Copy)]
enum TreeNode<'a> {
    Leaf {
        bound: BBox3<f64>,
        obj_index: u32,
        num_obj: u32,
    },
    Interior {
        bound: BBox3<f64>,
        children: (&'a TreeNode<'a>, &'a TreeNode<'a>),
        split_axis: u8,
    },
}

// #[repr(align(32))] <- experimental, TODO: add once not experimental
#[derive(Clone, Copy)]
enum LinearNodeKind {
    Leaf {
        obj_start_index: u32,
        obj_end_index: u32,
    },
    Interior {
        // left_child_index: it's always next to it in the array
        right_child_index: u32,
        split_axis: u8,
    },
}

#[derive(Clone, Copy)]
struct LinearNode {
    bound: BBox3<f64>,
    kind: LinearNodeKind,
}
