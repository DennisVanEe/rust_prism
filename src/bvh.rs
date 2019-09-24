use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::math::vector::Vec3;
use crate::geometry::Interaction;

use bumpalo::Bump;

pub trait BVHObject {
    // Any additional information for calculating the intersection:
    type IntParam;
    // Any additional information for calculatig the bounds or centroids:
    type DataParam;

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64, int_info: &Self::IntParam) -> bool;
    fn intersect(&self, ray: Ray<f64>, max_time: f64, int_info: &Self::IntParam) -> Option<Interaction>;

    fn get_centroid(&self, data: &Self::DataParam) -> Vec3<f64>;
    fn get_bound(&self, data: &Self::DataParam) -> BBox3<f64>;
}

pub struct BVH<O: BVHObject> {
    objects: Vec<O>,
    linear_nodes: Vec<LinearNode>,
    bound: BBox3<f64>,
}

impl<O: BVHObject> BVH<O> {
    // Constructs a BVH given a mesh and the max number of triangles per leaf node.
    // The BVH will become the owner of the mesh when doing this.
    pub fn new(objects: Vec<O>, max_obj_per_node: u32, data: &O::DataParam) -> Self {
        // First we record any object information we may need:
        let mut object_infos = Vec::with_capacity(objects.len());
        for (i, obj) in objects.iter().enumerate() {
            object_infos.push(ObjectInfo {
                object_index: i as u32,
                centroid: obj.get_centroid(data),
                bound: obj.get_bound(data),
            });
        }

        // Now we can go ahead and construct the tree:
        Self::construct_tree(objects, object_infos, max_obj_per_node)
    }

    // Given a mesh, triangle info (as passed by new), and the number of triangles per node,
    // construct a tree:
    fn construct_tree(
        mut objects: Vec<O>,
        mut object_infos: Vec<ObjectInfo>,
        max_obj_per_node: u32,
    ) -> Self {
        // Used to allocate the nodes:
        let mut memory = Bump::new();
        // The new triangles that will replace the ones in Mesh (they will be ordered
        // in the correct manner):
        let mut new_objects = Vec::with_capacity(objects.len());

        // Construct the regular tree first (that isn't flat):
        let (root_node, bound) = Self::recursive_construct_tree(
            max_obj_per_node,
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

    // Recursively constructs the tree.
    // Returns a reference to the root node of the tree and the bound of the entire tree:
    fn recursive_construct_tree<'a>(
        max_obj_per_node: u32,           // The maximum number of triangles per node.
        objects: &Vec<O>,                // The mesh we are currently constructing a BVH for.
        object_infos: &mut [ObjectInfo], // The current slice of triangles we are working on.
        new_objects: &mut Vec<O>,        // The correct order for the new triangles we care about.
        memory: &'a mut Bump,            // Allocator used to allocate the nodes. The lifetime of the nodes is that of the allocator
    ) -> (&'a TreeNode<'a>, BBox3<f64>) {
        // A bound over all of the triangles we are currently working with:
        let all_bound = object_infos.iter().fold(BBox3::new(), |all_bound, tri_info| {
            all_bound.combine_bnd(tri_info.bound)
        });

        // If we only have one triangle, make a leaf:
        if object_infos.len() == 1 {
            new_objects.push(objects[object_infos[0].object_index as usize]);
            return (
                memory.alloc(TreeNode::Leaf {
                    bound: all_bound,
                    object_index: (new_objects.len() - 1) as u32,
                    num_tri: 1,
                }),
                all_bound,
            );
        }

        // Otherwise, we want to split the tree into smaller parts:

        // The bound covering all of the centroids (used for SAH BVH construction):
        let centroid_bound = object_infos
            .iter()
            .fold(BBox3::new(), |centroid_bound, object_info| {
                centroid_bound.combine_pnt(object_info.centroid)
            });

        // Now we want to split based on the largest dimension:
        let max_dim = centroid_bound.max_dim();

        // Check if the volume has volume 0, if so, then create a leaf node:
        if centroid_bound.pmax[max_dim] == centroid_bound.pmin[max_dim] {
            // Need to keep track of where we will be putting these triangles.
            let curr_object_index = new_objects.len() as u32;
            for object_info in object_infos.iter() {
                new_objects.push(objects[object_info.object_index as usize]);
            }
            // Allocate the a new leaf node and push it:
            return (
                memory.alloc(TreeNode::Leaf {
                    bound: all_bound,
                    object_index: curr_object_index,
                    num_tri: object_infos.len() as u32,
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
struct ObjectInfo {
    pub object_index: u32,
    pub centroid: Vec3<f64>,
    pub bound: BBox3<f64>,
}

// This is the internal representation we have when initially building the tree.
// We later "flatten" the tree for efficient traversal.
#[derive(Clone, Copy)]
enum TreeNode<'a> {
    Leaf {
        bound: BBox3<f64>,
        object_index: u32,
        num_tri: u32,
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