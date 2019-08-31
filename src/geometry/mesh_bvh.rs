// This file contains a BVH implementation specifically designed for a mesh.
// The BVH is the owner of the mesh itself, but is not a geometry object.

use crate::geometry::mesh::{Mesh, RayIntInfo, Triangle};
use crate::geometry::Interaction;
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::Vec3;
use crate::memory::stack_alloc::StackAlloc;

use order_stat::kth_by;
use partition::partition;

use std::mem::MaybeUninit;

pub struct MeshBVH {
    mesh: Mesh,                    // The mesh of the BVH (the BVH owns the mesh)
    linear_nodes: Vec<LinearNode>, // The nodes that make up the tree
    bound: BBox3<f64>,             // The overall bounding box of the entire BVH
}

impl MeshBVH {
    // Number of buckets used for SAH:
    const BUCKET_COUNT: usize = 12;
    const ALLOC_STACK_SIZE: usize = 1024 * 1024 / 32; // I might specify something else later

    // Constructs a BVH given a mesh and the max number of triangles per leaf node.
    // The BVH will become the owner of the mesh when doing this.
    pub fn new(mesh: Mesh, max_tri_per_node: u32) -> Self {
        // First we record any triangle information we may need:
        let tris_raw = mesh.get_tri_raw();
        let mut tris_info = Vec::with_capacity(tris_raw.len());
        for (i, tri) in tris_raw.iter().enumerate() {
            tris_info.push(TriangleInfo {
                tri_index: i as u32,
                centroid: tri.centroid(&mesh),
                bound: tri.bound(&mesh),
            });
        }

        // Now we can go ahead and construct the tree:
        Self::construct_tree(mesh, tris_info, max_tri_per_node)
    }

    pub fn object_bound(&self) -> BBox3<f64> {
        self.bound
    }

    pub fn intersect_test(&self, ray: Ray<f64>, max_time: f64, int_info: RayIntInfo) -> bool {
        // This function has to be very efficient, so I'll be using a lot of unsafe code
        // here (but everything I'm doing should still be defined behavior).

        let inv_dir = ray.dir.inv_scale(1.);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut node_stack: [usize; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut node_stack_index = 0usize;
        let mut curr_node_index = 0usize;

        loop {
            let curr_node = *unsafe { self.linear_nodes.get_unchecked(curr_node_index) };
            if curr_node
                .bound
                .intersect_test(ray, max_time, inv_dir, is_dir_neg)
            {
                match curr_node.kind {
                    LinearNodeKind::Leaf {
                        tri_start_index,
                        tri_end_index,
                    } => {
                        let tri_start = tri_start_index as usize;
                        let tri_end = tri_end_index as usize;
                        unsafe {
                            for tri in self
                                .mesh
                                .get_tri_raw()
                                .get_unchecked(tri_start..tri_end)
                                .iter()
                            {
                                if tri.intersect_test(ray, max_time, int_info, &self.mesh) {
                                    return true;
                                }
                            }
                        }

                        // Pop the stack (if it's empty, we are done):
                        if node_stack_index == 0usize {
                            return false;
                        }
                        node_stack_index -= 1;
                        curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
                    }
                    LinearNodeKind::Interior {
                        right_child_index,
                        split_axis,
                    } => {
                        // Check which child it's most likely to be:
                        if is_dir_neg[split_axis as usize] {
                            // Push the first child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                curr_node_index + 1;
                            node_stack_index += 1;
                            curr_node_index = right_child_index as usize;
                        } else {
                            // Push the second child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                right_child_index as usize;
                            node_stack_index += 1;
                            curr_node_index += 1; // the first child
                        }
                    }
                }
            // If we don't hit it, then we try another item from the stack:
            } else {
                if node_stack_index == 0usize {
                    return false;
                }
                node_stack_index -= 1;
                curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
            }
        }
    }

    pub fn intersect(
        &self,
        ray: Ray<f64>,
        mut max_time: f64,
        int_info: RayIntInfo,
    ) -> Option<Interaction> {
        // This function has to be very efficient, so I'll be using a lot of unsafe code
        // here (but everything I'm doing should still be defined behavior).

        let inv_dir = ray.dir.inv_scale(1.);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        let mut node_stack: [usize; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut node_stack_index = 0usize;
        let mut curr_node_index = 0usize;

        // This is the final result:
        let mut result = None;

        loop {
            let curr_node = *unsafe { self.linear_nodes.get_unchecked(curr_node_index) };
            if curr_node
                .bound
                .intersect_test(ray, max_time, inv_dir, is_dir_neg)
            {
                match curr_node.kind {
                    LinearNodeKind::Leaf {
                        tri_start_index,
                        tri_end_index,
                    } => {
                        let tri_start = tri_start_index as usize;
                        let tri_end = tri_end_index as usize;
                        unsafe {
                            for tri in self
                                .mesh
                                .get_tri_raw()
                                .get_unchecked(tri_start..tri_end)
                                .iter()
                            {
                                if let Some(intersection) =
                                    tri.intersect(ray, max_time, int_info, &self.mesh)
                                {
                                    // Update the max time for more efficient culling:
                                    max_time = intersection.time;
                                    // Can't return immediately, have to make sure this is the closest intersection
                                    result = Some(intersection);
                                }
                            }
                        }

                        // Pop the stack (if it's empty, we are done):
                        if node_stack_index == 0usize {
                            break;
                        }
                        node_stack_index -= 1;
                        curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
                    }
                    LinearNodeKind::Interior {
                        right_child_index,
                        split_axis,
                    } => {
                        // Check which child it's most likely to be:
                        if is_dir_neg[split_axis as usize] {
                            // Push the first child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                curr_node_index + 1;
                            node_stack_index += 1;
                            curr_node_index = right_child_index as usize;
                        } else {
                            // Push the second child onto the stack to perform later:
                            *unsafe { node_stack.get_unchecked_mut(node_stack_index) } =
                                right_child_index as usize;
                            node_stack_index += 1;
                            curr_node_index += 1; // the first child
                        }
                    }
                }
            // If we don't hit it, then we try another item from the stack:
            } else {
                if node_stack_index == 0usize {
                    break;
                }
                node_stack_index -= 1;
                curr_node_index = *unsafe { node_stack.get_unchecked(node_stack_index) };
            }
        }

        result
    }

    // Given a mesh, triangle info (as passed by new), and the number of triangles per node,
    // construct a tree:
    fn construct_tree(
        mut mesh: Mesh,
        mut tris_info: Vec<TriangleInfo>,
        max_tri_per_node: u32,
    ) -> Self {
        // It would probably make more sense to create a better allocator for nodes then by doing
        // it this way, that way we could maintain pointers instead.
        let allocator = StackAlloc::new(Self::ALLOC_STACK_SIZE);
        // The new triangles that will replace the ones in Mesh (they will be ordered
        // in the correct manner):
        let mut new_tris = Vec::with_capacity(mesh.num_tris() as usize);

        // Construct the regular tree first (that isn't flat):
        let (root_node, bound) = Self::recursive_construct_tree(
            max_tri_per_node,
            &mesh,
            &mut tris_info,
            &mut new_tris,
            &allocator,
        );

        // Repalce the trianlges in the mesh with the reordered triangles:
        mesh.update_tris(new_tris);
        // Now we flatten the nodes for better memory and performance later down the line:
        let linear_nodes = Self::flatten_tree(allocator.get_alloc_count(), root_node);

        MeshBVH {
            mesh,
            linear_nodes,
            bound,
        }
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
                    tri_index,
                    num_tri,
                } => {
                    linear_nodes.push(LinearNode {
                        bound,
                        kind: LinearNodeKind::Leaf {
                            tri_start_index: tri_index,
                            tri_end_index: tri_index + num_tri,
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

    // Recursively constructs the tree.
    // Returns a reference to the root node of the tree and the bound of the entire tree:
    fn recursive_construct_tree<'a>(
        max_tri_per_node: u32,          // The maximum number of triangles per node.
        mesh: &Mesh,                    // The mesh we are currently constructing a BVH for.
        tri_infos: &mut [TriangleInfo], // The current slice of triangles we are working on.
        new_tris: &mut Vec<Triangle>,   // The correct order for the new triangles we care about.
        allocator: &'a StackAlloc<TreeNode<'a>>, // Allocator used to allocate the nodes. The lifetime of the nodes is that of the allocator
    ) -> (&'a TreeNode<'a>, BBox3<f64>) {
        // A bound over all of the triangles we are currently working with:
        let all_bound = tri_infos.iter().fold(BBox3::new(), |all_bound, tri_info| {
            all_bound.combine_bnd(tri_info.bound)
        });

        // If we only have one triangle, make a leaf:
        if tri_infos.len() == 1 {
            new_tris.push(mesh.get_tri(tri_infos[0].tri_index));
            return (
                allocator.push(TreeNode::Leaf {
                    bound: all_bound,
                    tri_index: (new_tris.len() - 1) as u32,
                    num_tri: 1,
                }),
                all_bound,
            );
        }

        // Otherwise, we want to split the tree into smaller parts:

        // The bound covering all of the centroids (used for SAH BVH construction):
        let centroid_bound = tri_infos
            .iter()
            .fold(BBox3::new(), |centroid_bound, tri_info| {
                centroid_bound.combine_pnt(tri_info.centroid)
            });

        // Now we want to split based on the largest dimension:
        let max_dim = centroid_bound.max_dim();

        // Check if the volume has volume 0, if so, then create a leaf node:
        if centroid_bound.pmax[max_dim] == centroid_bound.pmin[max_dim] {
            // Need to keep track of where we will be putting these triangles.
            let curr_tri_index = new_tris.len() as u32;
            for tri_info in tri_infos.iter() {
                new_tris.push(mesh.get_tri(tri_info.tri_index));
            }
            // Allocate the a new leaf node and push it:
            return (
                allocator.push(TreeNode::Leaf {
                    bound: all_bound,
                    tri_index: curr_tri_index,
                    num_tri: tri_infos.len() as u32,
                }),
                all_bound,
            );
        }

        // Figure out how to split the elements:
        // If we have less than 4 triangles, just split it evenly:
        let (tri_infos_left, tri_infos_right) = if tri_infos.len() <= 4 {
            // kth_by is essentially nth_element from C++.
            // Here, we reorder the triangles based on the value of the centroid
            // in the maximum dimension (dim).
            let mid = tri_infos.len() / 2;
            kth_by(tri_infos, mid, |tri_info0, tri_info1| {
                tri_info0.centroid[max_dim]
                    .partial_cmp(&tri_info1.centroid[max_dim])
                    .unwrap()
            });
            // Split the array:
            tri_infos.split_at_mut(mid)
        } else {
            // Otherwise, we perform this split based on surface-area heuristics:
            let mut buckets = [Bucket {
                count: 0,
                bound: BBox3::new(),
            }; Self::BUCKET_COUNT];

            for tri_info in tri_infos.iter() {
                // Get an index into where we are among the buckets:
                let bucket_ratio = centroid_bound.offset(tri_info.centroid)[max_dim];
                let bucket_index = if bucket_ratio == 1. {
                    Self::BUCKET_COUNT - 1
                } else {
                    ((Self::BUCKET_COUNT as f64) * bucket_ratio) as usize
                };

                let curr_bucket = &mut buckets[bucket_index];
                curr_bucket.count += 1;
                curr_bucket.bound = curr_bucket.bound.combine_bnd(tri_info.bound);
            }

            // Iterate over everything backwards, but ignore the first element to get the right
            // surface area values:
            let mut right_sa = [0f64; Self::BUCKET_COUNT - 1];
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
            let mut costs = [0f64; Self::BUCKET_COUNT - 1];
            let total_sa = all_bound.surface_area();
            buckets[..(Self::BUCKET_COUNT - 1)].iter().enumerate().fold(
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
            if tri_infos.len() > (max_tri_per_node as usize) || min_cost < (tri_infos.len() as f64)
            {
                // Split (partition) based on bucket with min cost:
                partition(tri_infos, |tri_info| {
                    let bucket_ratio = centroid_bound.offset(tri_info.centroid)[max_dim];
                    let bucket_index = if bucket_ratio == 1. {
                        Self::BUCKET_COUNT - 1
                    } else {
                        ((Self::BUCKET_COUNT as f64) * bucket_ratio) as usize
                    };
                    bucket_index <= min_cost_index
                })
            } else {
                // Otherwise, it isn't worth it so continue the splitting process, so we
                // create a leaf here:
                let curr_tri_index = new_tris.len() as u32;
                for tri_info in tri_infos.iter() {
                    new_tris.push(mesh.get_tri(tri_info.tri_index));
                }
                return (
                    allocator.push(TreeNode::Leaf {
                        bound: all_bound,
                        tri_index: curr_tri_index,
                        num_tri: tri_infos.len() as u32,
                    }),
                    all_bound,
                );
            }
        };

        // Build the left and right nodes now:
        let (left_node, _) = Self::recursive_construct_tree(
            max_tri_per_node,
            mesh,
            tri_infos_left,
            new_tris,
            allocator,
        );
        let (right_node, _) = Self::recursive_construct_tree(
            max_tri_per_node,
            mesh,
            tri_infos_right,
            new_tris,
            allocator,
        );

        // Create a node and push it on:
        (
            allocator.push(TreeNode::Interior {
                bound: all_bound,
                children: (left_node, right_node),
                split_axis: max_dim as u8,
            }),
            all_bound,
        )
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
struct TriangleInfo {
    pub tri_index: u32,
    pub centroid: Vec3<f64>,
    pub bound: BBox3<f64>,
}

// This is the internal representation we have when initially building the tree.
// We later "flatten" the tree for efficient traversal.
#[derive(Clone, Copy)]
enum TreeNode<'a> {
    Leaf {
        bound: BBox3<f64>,
        tri_index: u32,
        num_tri: u32,
    },
    Interior {
        bound: BBox3<f64>,
        children: (&'a TreeNode<'a>, &'a TreeNode<'a>),
        split_axis: u8,
    },
}

//#[repr(align(32))] <- experimental, TODOL: add once not experimental
#[derive(Clone, Copy)]
enum LinearNodeKind {
    Leaf {
        tri_start_index: u32,
        tri_end_index: u32,
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
