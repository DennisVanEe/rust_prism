use crate::geometry::mesh_bvh::MeshBVH;
use crate::geometry::{Interaction, Geometry};
use crate::transform::Transform;

pub struct MeshGeometry<T: Transform> {
    mesh: MeshBVH,

    geom_to_world: T,
    world_to_geom: T,
}

impl<T: Transform> Geometry for MeshGeometry<T> {
    // The bounds in geometry space:
    fn geom_bound(&self) -> BBox3<f64> {
        self.mesh.object_bound()
    }

    fn world_bound(&self, t: f64) -> BBox3<f64> {
        self.geom_to_world.bbox(self.geom_bound(), t)
    }

    fn intersect_test(&self, ray: Ray<f64>, max_time: f64) -> bool {

    }
    
    fn intersect(&self, ray: Ray<f64>, max_time: f64) -> Option<Interaction>;
}