// The BVH is used to efficiently intersect the mesh.

use crate::geometry::mesh::{Mesh, Triangle};
use crate::math::bbox::BBox3f;
use crate::math::vector::Vec3f;

use std::cmp::Ordering;

pub struct MeshBVH {
    mesh: Mesh,             // The mesh of the BVH (the BVH owns the mesh)
    nodes: Vec<LinearNode>, // The nodes that make up the tree
    tri_per_node: u32,      // the maximum number of triangles per node
}

// Node that is stored in contigious memory for efficient traversal:
#[repr(align(32))]
struct LinearNode {
    pub bound: BBox3f,
    // Either the index into the triangles array, or the index of the children,
    // depends on the value of the number of children:
    pub index: u32,
    pub num_tris: u16,
    pub axis: u8,
}

// Nodes used in the tree when constructing the MeshBVH. These nodes aren't used
// in the final representation of the tree.
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
struct TriangleInfo {
    pub tri_index: u32,
    pub centroid: Vec3f,
    pub bound: BBox3f,
}

impl MeshBVH {
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
                all_bound.combine_bnd(&tri_info.bound)
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
            
        }

        0usize
    }
}
