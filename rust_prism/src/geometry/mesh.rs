use crate::geometry::{GeomInteraction, Geometry};
use crate::transform::Transf;
use embree;
use math;
use math::ray::Ray;
use math::vector::{Vec2, Vec3};
use simple_error::SimpleResult;
use std::mem;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    pub fn calc_interaction(
        self,
        ray: Ray<f64>,         // The ray that led to the intersection
        hit: embree::Hit<f64>, // Embree's hit information
        mesh: &Mesh,           // A reference to the mesh being intersected
        material_id: u32,      // The material id of the intersection
    ) -> GeomInteraction {
        // Calculate the barycentric coordinate:
        let b = {
            let u = hit.uv.x as f64;
            let v = hit.uv.y as f64;
            [u, v, 1.0 - u - v]
        };

        // Calculate the geometric normal:
        let n = hit.ng.normalize();

        let t = ray.t_far;

        let wo = -ray.dir;

        // Get the poss information:
        let poss = self.pos(mesh);
        let p = poss[0].scale(b[0]) + poss[1].scale(b[1]) + poss[2].scale(b[2]);

        // Check if uvs are present or not:
        let uvs = if mesh.uvs.is_empty() {
            [
                Vec2 { x: 0.0, y: 0.0 },
                Vec2 { x: 1.0, y: 0.0 },
                Vec2 { x: 1.0, y: 1.0 },
            ]
        } else {
            // Because we are gauranteed that nrm isn't empty:
            self.uvs(mesh)
        };

        let uv = uvs[0].scale(b[0]) + uvs[1].scale(b[1]) + uvs[2].scale(b[2]);

        let dp02 = poss[0] - poss[2];
        let dp12 = poss[1] - poss[2];
        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];

        // Calculate the derivative components:
        let det = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let inv_det = 1.0 / det;
        let dpdu = (dp02.scale(duv12[1]) - dp12.scale(duv02[1])).scale(inv_det);
        let dpdv = (dp02.scale(-duv12[0]) + dp12.scale(duv02[0])).scale(inv_det);

        // Calculate normal shading information:
        let (n, shading_n, shading_dndu, shading_dndv) = if mesh.nrm.is_empty() {
            (n, n, Vec3::zero(), Vec3::zero())
        } else {
            let nrms = self.nrm(mesh);
            let shading_n = nrms[0].scale(b[0]) + nrms[1].scale(b[1]) + nrms[2].scale(b[2]);
            let dn02 = nrms[0] - nrms[2];
            let dn12 = nrms[1] - nrms[2];
            let dndu = (dn02.scale(duv12[1]) - dn12.scale(duv02[1])).scale(inv_det);
            let dndv = (dn02.scale(-duv12[0]) + dn12.scale(duv02[0])).scale(inv_det);
            let n = math::align(shading_n, n); // If we have shading normals, let is decide orientation
            (n, shading_n, dndu, dndv)
        };

        // Calculate tangent shading informatin:
        let shading_dpdu = if mesh.tan.is_empty() {
            dpdu.normalize()
        } else {
            let tans = self.tan(mesh);
            tans[0].scale(b[0]) + tans[1].scale(b[1]) + tans[2].scale(b[2])
        };

        let shading_dpdv = shading_n.cross(shading_dpdu).normalize();
        let shading_dpdu = shading_dpdv.cross(shading_n);

        GeomInteraction {
            p,
            n,
            wo,
            t,
            uv,
            dpdu,
            dpdv,
            shading_n,
            shading_dpdu,
            shading_dpdv,
            shading_dndu,
            shading_dndv,
            material_id,
        }
    }

    pub fn area(self, mesh: &Mesh) -> f64 {
        let pos = self.pos(mesh);
        let a = pos[1] - pos[0];
        let b = pos[2] - pos[0];
        a.cross(b).length() * 0.5
    }

    pub fn pos(self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        [
            mesh.pos[self.indices[0] as usize].to_f64(),
            mesh.pos[self.indices[1] as usize].to_f64(),
            mesh.pos[self.indices[2] as usize].to_f64(),
        ]
    }

    pub fn nrm(self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        [
            mesh.nrm[self.indices[0] as usize].to_f64(),
            mesh.nrm[self.indices[1] as usize].to_f64(),
            mesh.nrm[self.indices[2] as usize].to_f64(),
        ]
    }

    pub fn tan(self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        [
            mesh.tan[self.indices[0] as usize].to_f64(),
            mesh.tan[self.indices[1] as usize].to_f64(),
            mesh.tan[self.indices[2] as usize].to_f64(),
        ]
    }

    pub fn uvs(self, mesh: &Mesh) -> [Vec2<f64>; 3] {
        [
            mesh.uvs[self.indices[0] as usize].to_f64(),
            mesh.uvs[self.indices[1] as usize].to_f64(),
            mesh.uvs[self.indices[2] as usize].to_f64(),
        ]
    }
}
pub struct Mesh {
    pub triangles: Vec<Triangle>,

    pub pos: Vec<Vec3<f32>>,
    pub nrm: Vec<Vec3<f32>>,
    pub tan: Vec<Vec3<f32>>,
    pub uvs: Vec<Vec2<f32>>,

    // The surface area of the mesh.
    surface_area: f64,

    // A pointer to the geometry in embree.
    embree_geom: embree::Geometry,
}

impl Mesh {
    pub fn new(
        triangles: Vec<Triangle>,
        pos: Vec<Vec3<f32>>,
        nrm: Vec<Vec3<f32>>,
        tan: Vec<Vec3<f32>>,
        uvs: Vec<Vec2<f32>>,
    ) -> Self {
        Mesh {
            triangles,
            pos,
            nrm,
            tan,
            uvs,
            surface_area: -1.0,
            embree_geom: embree::Geometry::new_null(),
        }
    }
}

impl Geometry for Mesh {
    /// Permanently applies the transformation to the data of the mesh.
    fn transform(&mut self, transf: Transf) {
        transf.points_f32(&mut self.pos);
        transf.vectors_f32(&mut self.tan);
        transf.normals_f32(&mut self.nrm);
    }

    /// Creates the embree geometry for this specific mesh.
    ///
    /// Creates the embree geometry. Can be called multiple times, will only create
    /// the geometry once.
    fn create_embree_geometry(&mut self, device: embree::Device) -> SimpleResult<embree::Geometry> {
        // Delete the device first.
        self.delete_embree_geometry(device)?;

        let embree_geom = embree::new_geometry(device, embree::GeometryType::Triangle)?;
        embree::set_shared_geometry_buffer(
            device,
            embree_geom,
            embree::BufferType::Vertex,
            0,
            embree::Format::Float3,
            self.pos.as_ptr(),
            0,
            mem::size_of::<Vec3<f32>>(),
            self.pos.len() - 1, // See why we allocate one extra pos at the end
        )?;
        embree::set_shared_geometry_buffer(
            device,
            embree_geom,
            embree::BufferType::Index,
            0,
            embree::Format::Uint3,
            self.triangles.as_ptr(),
            0,
            mem::size_of::<Triangle>(),
            self.triangles.len(),
        )?;

        self.embree_geom = embree_geom;
        Ok(embree_geom)
    }

    fn delete_embree_geometry(&mut self, device: embree::Device) -> SimpleResult<()> {
        if self.embree_geom.is_null() {
            return Ok(());
        }

        let result = embree::release_geometry(device, self.embree_geom);
        self.embree_geom = embree::Geometry::new_null();
        result
    }

    /// Returns the current RTCGeometry.
    fn get_embree_geometry(&self) -> embree::Geometry {
        self.embree_geom
    }

    fn get_surface_area(&self) -> f64 {
        self.surface_area
    }

    /// Calculates the surface area of the specific mesh.
    fn calc_surface_area(&mut self) -> f64 {
        if self.surface_area >= 0.0 {
            return self.surface_area;
        }

        //let mesh_ref = self.get_ref();
        self.surface_area = self
            .triangles
            .iter()
            .fold(0.0, |sa, triangle| sa + triangle.area(self));
        self.surface_area
    }

    fn calc_interaction(
        &self,
        ray: Ray<f64>,
        hit: embree::Hit<f64>,
        material_id: u32,
    ) -> GeomInteraction {
        // Get the primitive:
        let triangle = self.triangles[hit.prim_id as usize];
        triangle.calc_interaction(ray, hit, self, material_id)
    }
}
