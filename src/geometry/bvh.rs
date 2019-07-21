// The BVH is used to efficiently intersect the mesh.

use crate::geometry::mesh::{Mesh, MeshData, Triangle};
use crate::math::bbox::BBox3f;

pub struct MeshBVH {
    mesh: Mesh,

    
    mesh: MeshData,
    nodes: Vec<LinearNode>,
}

// Node that is stored in contigious memory for efficient traversal:
#[repr(align(32))]
struct LinearNode {
    bound: BBox3f,
    // Either the index into the triangles array, or the index of the children,
    // depends on the value of the number of children:
    index: u32,
    num_tris: u16,
    axis: u8,
}