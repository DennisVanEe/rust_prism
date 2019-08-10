// The BVH is used to efficiently intersect the mesh.

use crate::geometry::mesh::{Intersection, Mesh, RayIntInfo, Triangle};
use crate::math::bbox::BBox3f;
use crate::math::ray::Ray;
use crate::math::vector::Vec3f;
use crate::memory::stack_alloc::StackAlloc;

use arrayvec::ArrayVec;
use order_stat::kth_by;
use partition::partition;

pub struct MeshBVH {
    mesh: Mesh,                    // The mesh of the BVH (the BVH owns the mesh)
    linear_nodes: Vec<LinearNode>, // The nodes that make up the tree
}

impl MeshBVH {
    // Number of buckets used for SAH:
    const BUCKET_COUNT: usize = 12;

    //
    // MeshBVH Public Interface
    //

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

    pub fn intersect_test(&self, ray: Ray, mut max_time: f32, int_info: RayIntInfo) -> Option<f32> {
        let mut curr_node = match self.linear_nodes.first() {
            Some(&val) => val,
            None => return None,
        };

        // Some values we would need before we start:
        let inv_dir = ray.dir.inv_scale(1f32);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        // We could do this recursively, but a loop is more efficient in this case:

        // This is the stack used to traverse the tree:
        let mut node_index_stack = ArrayVec::<[usize; 64]>::new();
        let mut curr_node_index = 0usize;
        loop {
            // We don't care where our ray intersects in time:
            if let Some(_) = curr_node
                .bound
                .intersect_test(ray, max_time, inv_dir, is_dir_neg)
            {
                // Check if it is a leaf node (and thus, we can traverse the nodes):
                if curr_node.num_tri > 0 {
                    // Traverse over the triangles we want to check an intersection for:
                    let begin = curr_node.tri_index as usize;
                    let end = begin + (curr_node.num_tri as usize);
                    let triangles = &self.mesh.get_tri_raw()[begin..end];

                    for tri in triangles.iter() {
                        max_time = match tri.intersect_test(ray, max_time, int_info, &self.mesh) {
                            Some(time) => time,
                            None => return None,
                        };
                    }

                    // Pop the stack:
                    curr_node_index = match node_index_stack.pop() {
                        Some(val) => val,
                        None => return None,
                    } as usize;
                    // We can do this because we are guaranteed the algorithm works:
                    curr_node = unsafe { *self.linear_nodes.get_unchecked(curr_node_index) };
                } else {
                    // Check which child it's most likely to be:
                    if is_dir_neg[curr_node.split_axis as usize] {
                        // Push the first child onto the stack to perform later:
                        node_index_stack.push(curr_node_index + 1);
                        // Get the second child (unsafe because it's gauranteed to work):
                        curr_node = unsafe {
                            *self
                                .linear_nodes
                                .get_unchecked(curr_node.tri_index as usize)
                        };
                    } else {
                        // Push the second child onto the stack to perform later:
                        node_index_stack.push(curr_node.tri_index as usize);
                        // Get the first child (unsafe because it's gauranteed to work):
                        curr_node =
                            unsafe { *self.linear_nodes.get_unchecked(curr_node_index + 1) };
                    }
                }
            } else {
                // Pop the stack:
                curr_node_index = match node_index_stack.pop() {
                    Some(val) => val,
                    None => return None,
                } as usize;
                // We can do this because we are guaranteed the algorithm works:
                curr_node = unsafe { *self.linear_nodes.get_unchecked(curr_node_index) };
            }
        }

        None
    }

    pub fn intersect(
        &self,
        mut max_time: f32,
        ray: Ray,
        int_info: RayIntInfo,
    ) -> Option<Intersection> {
        let mut curr_node = match self.linear_nodes.first() {
            Some(&val) => val,
            None => return None,
        };

        // Some values we would need before we start:
        let inv_dir = ray.dir.inv_scale(1f32);
        let is_dir_neg = ray.dir.comp_wise_is_neg();

        // We could do this recursively, but a loop is more efficient in this case:

        // This is the stack used to traverse the tree:
        let mut node_index_stack = ArrayVec::<[usize; 64]>::new();
        let mut curr_node_index = 0usize;
        let mut intersection = None;
        loop {
            // We don't care where our ray intersects in time:
            if let Some(_) = curr_node
                .bound
                .intersect_test(ray, max_time, inv_dir, is_dir_neg)
            {
                // Check if it is a leaf node (and thus, we can traverse the nodes):
                if curr_node.num_tri > 0 {
                    // Traverse over the triangles we want to check an intersection for:
                    let begin = curr_node.tri_index as usize;
                    let end = begin + (curr_node.num_tri as usize);
                    let triangles = &self.mesh.get_tri_raw()[begin..end];

                    // Update the hit and the max_time (so we can ignore values that are too far).
                    // Unlike before, we can't return instantly, because there might be a closer intersection.
                    for tri in triangles.iter() {
                        if let Some(int) = tri.intersect(ray, max_time, int_info, &self.mesh) {
                            intersection = Some(int);
                            max_time = int.time;
                        }
                    }

                    // Pop the stack:
                    curr_node_index = match node_index_stack.pop() {
                        Some(val) => val,
                        None => return None,
                    } as usize;
                    // We can do this because we are guaranteed the algorithm works:
                    curr_node = unsafe { *self.linear_nodes.get_unchecked(curr_node_index) };
                } else {
                    // Check which child it's most likely to be:
                    if is_dir_neg[curr_node.split_axis as usize] {
                        // Push the first child onto the stack to perform later:
                        node_index_stack.push(curr_node_index + 1);
                        // Get the second child (unsafe because it's gauranteed to work):
                        curr_node = unsafe {
                            *self
                                .linear_nodes
                                .get_unchecked(curr_node.tri_index as usize)
                        };
                    } else {
                        // Push the second child onto the stack to perform later:
                        node_index_stack.push(curr_node.tri_index as usize);
                        // Get the first child (unsafe because it's gauranteed to work):
                        curr_node =
                            unsafe { *self.linear_nodes.get_unchecked(curr_node_index + 1) };
                    }
                }
            } else {
                // Pop the stack:
                curr_node_index = match node_index_stack.pop() {
                    Some(val) => val,
                    None => return None,
                } as usize;
                // We can do this because we are guaranteed the algorithm works:
                curr_node = unsafe { *self.linear_nodes.get_unchecked(curr_node_index) };
            }
        }

        // If we were lucky enough to hit something, it'll be returned here:
        intersection
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
        let mut allocator = StackAlloc::new(32768);
        // The new triangles that will replace the ones in Mesh (they will be ordered
        // in the correct manner):
        let mut new_tris = Vec::with_capacity(mesh.num_tris() as usize);

        // Construct the regular tree first:
        let root_node = Self::recursive_construct_tree(
            max_tri_per_node,
            &mesh,
            &mut tris_info,
            &mut new_tris,
            &mut allocator,
        );

        // Now construct the linear tree (which should be more efficient):
        // We already know how many nodes we will be allocating:
        let mut linear_nodes = Vec::with_capacity(allocator.get_num_alloc());
        // Use the new triangles in their better position:
        mesh.update_tris(new_tris);
        // Create the linear nodes:
        //Self::flatten_tree(&mut linear_nodes, root_node);
        MeshBVH { mesh, linear_nodes }
    }

    // Given a tree represented as a "tree node", we can "flatten" it in a specific manner that
    // would allow for more efficient traversal (technically, the tree_nodes are already flat in
    // the sense that they are contigiously in memory).
    fn flatten_tree(
        linear_nodes: &mut Vec<LinearNode>,
        curr_tree_node: &TreeNode,
    ) -> usize {
        match curr_tree_node.children {
            // There are no children, so it must be a leaf:
            None => {
                linear_nodes.push(LinearNode::create_leaf(
                    curr_tree_node.tri_index,
                    curr_tree_node.num_tri,
                    curr_tree_node.bound,
                ));
                linear_nodes.len() - 1
            }
            // There are children, so it must be a leaf:
            Some((left_child, right_child)) => {
                let curr_pos = linear_nodes.len();
                linear_nodes.push(LinearNode::create_interior(
                    curr_tree_node.split_axis,
                    0,
                    curr_tree_node.bound,
                ));
                Self::flatten_tree(linear_nodes, left_child);
                linear_nodes[curr_pos].tri_index =
                    Self::flatten_tree(linear_nodes, right_child)
                        as u32;
                curr_pos
            }
        }
    }

    // Recursively constructs the tree:
    // Returns the index of the node created in the next call, that node will be on the vector
    // That one passes to it.
    fn recursive_construct_tree<'a>(
        max_tri_per_node: u32,                 // The maximum number of triangles per node.
        mesh: &Mesh,                           // The mesh we are currently constructing a BVH for.
        tri_infos: &mut [TriangleInfo],        // The current slice of triangles we are working on.
        new_tris: &mut Vec<Triangle>,          // The correct order for the new triangles we care about.
        allocator: &'a mut StackAlloc<TreeNode<'a>>,  // Allocator used to allocate the nodes.
    ) -> &'a TreeNode<'a> {
        // A bound over all of the triangles we are currently working with:
        let all_bound = tri_infos.iter().fold(BBox3f::new(), |all_bound, tri_info| {
            all_bound.combine_bnd(tri_info.bound)
        });

        // If we only have one triangle, make a leaf:
        if tri_infos.len() == 1 {
            new_tris.push(mesh.get_tri(tri_infos[0].tri_index));
            return allocator.push(TreeNode::create_leaf(all_bound, (new_tris.len() - 1) as u32, 1));
        }

        // Otherwise, we want to split the tree into smaller parts:

        // The bound covering all of the centroids (used for SAH BVH construction):
        let centroid_bound = tri_infos
            .iter()
            .fold(BBox3f::new(), |centroid_bound, tri_info| {
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
            return allocator.push(TreeNode::create_leaf(
                all_bound,
                curr_tri_index,
                tri_infos.len() as u32,
            ));
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
                bound: BBox3f::new(),
            }; Self::BUCKET_COUNT];

            for tri_info in tri_infos.iter() {
                // Get an index into where we are among the buckets:
                let bucket_ratio = centroid_bound.offset(tri_info.centroid)[max_dim];
                let bucket_index = if bucket_ratio == 1f32 {
                    Self::BUCKET_COUNT - 1
                } else {
                    ((Self::BUCKET_COUNT as f32) * bucket_ratio) as usize
                };

                let curr_bucket = &mut buckets[bucket_index];
                curr_bucket.count = curr_bucket.count + 1;
                curr_bucket.bound = curr_bucket.bound.combine_bnd(tri_info.bound);
            }

            // Iterate over everything backwards, but ignore the first element to get the right
            // surface area values:
            let mut right_sa = [0f32; Self::BUCKET_COUNT - 1];
            let (right_bound, right_count) = buckets[1..].iter().enumerate().rev().fold(
                (BBox3f::new(), 0u32),
                |(right_bound, right_count), (i, bucket)| {
                    // Have to do this because enumerate starts at 0, always, not the index of the slice:
                    let i = i + 1;
                    let right_bound = right_bound.combine_bnd(bucket.bound);
                    right_sa[i - 1] = right_bound.surface_area();
                    let right_count = right_count + bucket.count;
                    (right_bound, right_count)
                },
            );

            // Now we can compute the values going forward to fill in the buckets.
            // We also must modify the right count as we decrement it over time:
            let mut costs = [0f32; Self::BUCKET_COUNT - 1];
            let total_sa = all_bound.surface_area();
            let (left_bound, left_count, _) =
                buckets[..Self::BUCKET_COUNT - 1].iter().enumerate().fold(
                    (BBox3f::new(), 0u32, right_count),
                    |(left_bound, left_count, right_count), (i, bucket)| {
                        let left_bound = left_bound.combine_bnd(bucket.bound);
                        let left_count = left_count + bucket.count;
                        // Calculate the heuristic here:
                        costs[i] = 0.125f32
                            * ((left_count as f32) * left_bound.surface_area()
                                + (right_count as f32) * right_sa[i])
                            / total_sa;
                        let right_count = right_count - buckets[i + 1].count;
                        (left_bound, left_count, right_count)
                    },
                );

            let (min_cost_index, &min_cost) = costs
                .iter() // returns a reference to the elements (so a &x essentially).
                .enumerate() // returns (i, &x), and max_by's lambda takes a reference. But coercion helps here:
                .max_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
                .unwrap();

            // If this happens, then we should split more and continue our operations:
            if tri_infos.len() > (max_tri_per_node as usize) || min_cost < (tri_infos.len() as f32)
            {
                // Split (partition) based on bucket with min cost:
                partition(tri_infos, |tri_info| {
                    let bucket_ratio = centroid_bound.offset(tri_info.centroid)[max_dim];
                    let bucket_index = if bucket_ratio == 1f32 {
                        Self::BUCKET_COUNT - 1
                    } else {
                        ((Self::BUCKET_COUNT as f32) * bucket_ratio) as usize
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
                return allocator.push(TreeNode::create_leaf(
                    all_bound,
                    curr_tri_index,
                    tri_infos.len() as u32,
                ));
            }
        };

        // Build the left and right nodes now:
        let left_node = Self::recursive_construct_tree(
            max_tri_per_node,
            mesh,
            tri_infos_left,
            new_tris,
            allocator,
        );
        let right_node = Self::recursive_construct_tree(
            max_tri_per_node,
            mesh,
            tri_infos_right,
            new_tris,
            allocator,
        );

        // Create a node and push it on:
        allocator.push(TreeNode::create_interior(
            max_dim as u8,
            left_node,
            right_node,
        ))
    }
}

// Align to 32 bytes to make it more cache-friendly:
#[repr(align(32))]
#[derive(Clone, Copy)]
struct LinearNode {
    pub bound: BBox3f,
    // Either the index into the triangles array, or the index of the second child,
    // depends on the value of the number of children:
    pub tri_index: u32,
    pub num_tri: u32,
    pub split_axis: u8,
}

impl LinearNode {
    pub fn create_leaf(tri_index: u32, num_tri: u32, bound: BBox3f) -> Self {
        LinearNode {
            tri_index,
            num_tri,
            bound,
            split_axis: 0,
        }
    }

    pub fn create_interior(split_axis: u8, tri_index: u32, bound: BBox3f) -> Self {
        LinearNode {
            split_axis,
            tri_index,
            bound,
            num_tri: 0,
        }
    }
}

// This is the bucket used for SAH splitting:
#[derive(Clone, Copy)]
struct Bucket {
    // Number of items in the current bucket:
    pub count: u32,
    // Bound for the current bucket:
    pub bound: BBox3f,
}

// This is a node used for the "unflattened" tree.
#[derive(Clone, Copy)]
struct TreeNode<'a> {
    pub bound: BBox3f,
    // children that index into the nodes vector
    pub children: Option<(&'a TreeNode<'a>, &'a TreeNode<'a>)>,

    // index into the triangle array now used:
    pub tri_index: u32,
    pub num_tri: u32,
    pub split_axis: u8,
}

impl<'a> TreeNode<'a> {
    // Simple stuff we care about:
    pub fn create_leaf(bound: BBox3f, tri_index: u32, num_tri: u32) -> Self {
        TreeNode {
            bound,
            tri_index,
            num_tri,
            children: None,
            split_axis: 0,
        }
    }

    pub fn create_interior(split_axis: u8, left: &'a TreeNode<'a>, right: &'a TreeNode<'a>) -> Self {
        TreeNode {
            split_axis,
            children: Some((left, right)),
            bound: left.bound.combine_bnd(right.bound),
            tri_index: 0,
            num_tri: 0,
        }
    }
}

// Structure used to construct the BVH:
#[derive(Clone, Copy)]
struct TriangleInfo {
    pub tri_index: u32,
    pub centroid: Vec3f,
    pub bound: BBox3f,
}
