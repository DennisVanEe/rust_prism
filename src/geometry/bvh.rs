// The BVH is used to efficiently intersect the mesh.

use crate::geometry::mesh::{Mesh, Triangle};
use crate::math::bbox::BBox3f;
use crate::math::vector::Vec3f;

use std::cmp::Ordering;

pub struct MeshBVH {
    mesh: Mesh,             // The mesh of the BVH (the BVH owns the mesh)
    nodes: Vec<LinearNode>, // The nodes that make up the tree
    max_tri_per_node: u32,  // the maximum number of triangles per node
}

impl MeshBVH {
    // Number of buckets used for SAH:
    const BUCKET_COUNT: usize = 12;

    // Constructs a BVH given a mesh and the number of nodes:
    pub fn new(mesh: Mesh, tri_per_node: u32) -> Self {
        // First we construct information about the triangles:
        let mut tris_info = Vec::with_capacity(mesh.num_tris() as usize);
        for i in 0..mesh.num_tris() {
            tris_info.push(TriangleInfo {
                tri_index: i,
                centroid: mesh.get_tri(i).centroid(&mesh),
                bound: mesh.get_tri(i).bound(&mesh),
            });
        }

        Self::construct_tree(mesh, &mut tris_info)
    }

    fn construct_tree(mut mesh: Mesh, tris_info: &mut Vec<TriangleInfo>) -> Self {
        let mut tree_nodes = Vec::new();
        // The new triangles that will replace the ones in Mesh:
        let mut new_tris = Vec::with_capacity(mesh.num_tris() as usize);
    }

    // Recursively constructs the tree:
    // Returns the index of the node created in the next call
    fn recursive_construct_tree(
        max_tri_per_node: u32,
        start: usize,
        end: usize,
        mesh: &Mesh,
        tri_infos: &mut Vec<TriangleInfo>,
        new_tris: &mut Vec<Triangle>,
        tree_nodes: &mut Vec<TreeNode>,
    ) -> usize {
        // The current triangles we are working with:
        let curr_tri_infos = &mut tri_infos[start..end];
        let all_bound = curr_tri_infos
            .iter()
            .fold(BBox3f::new(), |all_bound, tri_info| {
                all_bound.combine_bnd(tri_info.bound)
            });

        let num_tris = end - start;

        // If we only have one triangle, make a leaf:
        if num_tris == 1 {
            tree_nodes.push(TreeNode::create_leaf(all_bound, new_tris.len() as u32, 1));
            new_tris.push(mesh.get_tri(curr_tri_infos[0].tri_index));
            return tree_nodes.len() - 1;
        }

        //
        // Otherwise, we may have to perform some splitting:
        //

        // The bound covering all of the centroids (used for SAH BVH construction):
        let centroid_bound = curr_tri_infos
            .iter()
            .fold(BBox3f::new(), |centroid_bound, tri_info| {
                centroid_bound.combine_pnt(tri_info.centroid)
            });

        // Now we want to split an axis:
        let dim = centroid_bound.max_dim();

        // Check if the volume has volume 0
        // ( we are done then if this is the case, create a leaf):
        if centroid_bound.pmax[dim] == centroid_bound.pmin[dim] {
            let curr_tri_index = new_tris.len() as u32;
            for tri_info in curr_tri_infos {
                new_tris.push(mesh.get_tri(tri_info.tri_index));
            }
            tree_nodes.push(TreeNode::create_leaf(
                all_bound,
                curr_tri_index,
                num_tris as u32,
            ));
            return tree_nodes.len() - 1;
        }

        let mid = (start + end) / 2;

        // Figure out how to split the elements:
        if num_tris <= 4 {
            // similar to "nth element" from C++
            // Essentially, we split the elements equally midway:
            curr_tri_infos.partition_at_index_by(mid, |tri_info0, tri_info1| {
                tri_info0.centroid[dim]
                    .partial_cmp(&tri_info1.centroid[dim])
                    .unwrap()
            });
        } else {
            // Otherwise, we perform this split based on surface-area heuristics:
            let mut buckets = [ Bucket { count: 0, bound: BBox3f::new() }; Self::BUCKET_COUNT ];
            for tri_info in curr_tri_infos {
                // Get an index into where we are among the buckets:
                let bucket_ratio = centroid_bound.offset(tri_info.centroid)[dim];
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
            let mut right_sa = [ 0f32; Self::BUCKET_COUNT - 1 ];
            let (right_bound, right_count) = buckets[1..]
                .iter()
                .rev()
                .enumerate()
                .fold((BBox3f::new(), 0u32), |(right_bound, right_count), (i, bucket)| {
                    let right_bound = right_bound.combine_bnd(bucket.bound);
                    right_sa[i - 1] = right_bound.surface_area();
                    let right_count = right_count + bucket.count;
                    (right_bound, right_count)
                });
            
            // Now we can compute the values going forward to fill in the buckets.
            // We also must modify the right count as we decrement it over time:
            let mut costs = [ 0f32; Self::BUCKET_COUNT - 1 ];
            let total_sa = all_bound.surface_area();
            let (left_bound, left_count, _) = buckets[..Self::BUCKET_COUNT - 1]
                .iter()
                .enumerate()
                .fold((BBox3f::new(), 0u32, right_count), |(left_bound, left_count, right_count), (i, bucket)| {
                    let left_bound = left_bound.combine_bnd(bucket.bound);
                    let left_count = left_count + bucket.count;
                    // Calculate the heuristic here:
                    costs[i] = 0.125f32 * ((left_count as f32) * left_bound.surface_area() + 
                        (right_count as f32) * right_sa[i]) / total_sa;
                    let right_count = right_count - buckets[i + 1].count;
                    (left_bound, left_count, right_count)
                });

            let (min_cost_index, &min_cost) = costs
                .iter() // returns a reference to the elements
                .enumerate() // returns (i, &x), and max_by's lambda takes a reference. But coercion helps here:
                .max_by(|(_, x), (_, y)| { x.partial_cmp(y).unwrap() } ).unwrap();

            // If this happens, then we should split more and continue our operations:
            if num_tris > (max_tri_per_node as usize) || min_cost < (num_tris as f32) {
                curr_tri_infos.partition_at_index_by(mid, |tri_info0, tri_info1| {
                tri_info0.centroid[dim]
                    .partial_cmp(&tri_info1.centroid[dim])
                    .unwrap()
                });
            }
        }

        0usize
    }
}

// Node that is stored in contigious memory for efficient traversal:
#[repr(align(32))]
#[derive(Clone, Copy)]
struct LinearNode {
    pub bound: BBox3f,
    // Either the index into the triangles array, or the index of the children,
    // depends on the value of the number of children:
    pub index: u32,
    pub num_tris: u16,
    pub axis: u8,
}

// This is the bucket used for SAH splitting:
#[derive(Clone, Copy)]
struct Bucket {
    // Number of items in the current bucket:
    pub count: u32,
    // Bound for the current bucket:
    pub bound: BBox3f,
}

// Nodes used in the tree when constructing the MeshBVH. These nodes aren't used
// in the final representation of the tree.
#[derive(Clone, Copy)]
struct TreeNode {
    pub bound: BBox3f,
    // children that index into the nodes vector
    pub children: Option<[u32; 2]>,

    pub split_axis: u8,
    pub tri_index: u32,
    pub num_tri: u32,
}

impl TreeNode {
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
}

// Internally used structure that represents information about a triangle
// (we only use this temporarily):
#[derive(Clone, Copy)]
struct TriangleInfo {
    pub tri_index: u32,
    pub centroid: Vec3f,
    pub bound: BBox3f,
}
