use crate::device::Device;
use crate::math::util;
use crate::math::vector::{Vec2, Vec3};
use crate::transform::Transform;
use embree;
use simple_error::SimpleResult;
use std::cell::Cell;
use std::mem;
use std::os::raw;
use std::ptr;

#[derive(Clone, Copy)]
pub struct Interaction {
    pub p: Vec3<f64>,  // intersection point
    pub n: Vec3<f64>,  // geometric normal (of triangle)
    pub wo: Vec3<f64>, // direction of intersection leaving the point

    pub t: f64, // the t value of the intersection of the ray

    pub uv: Vec2<f64>,   // uv coordinate at the intersection
    pub dpdu: Vec3<f64>, // vectors parallel to the triangle
    pub dpdv: Vec3<f64>,

    pub shading_n: Vec3<f64>,    // the shading normal at this point
    pub shading_dpdu: Vec3<f64>, // the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3<f64>,
    pub shading_dndu: Vec3<f64>, // the shading dndu, dndv at this point
    pub shading_dndv: Vec3<f64>,

    pub material_id: u32, // An index to the material specified with this interaction
}

#[derive(Clone, Copy)]
pub struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    pub fn calc_interaction(
        self,
        rayhit: embree::RTCRayHit,
        mesh: &Mesh,
        material_id: u32,
    ) -> Interaction {
        // Calculate the barycentric coordinate:
        let b = {
            let u = rayhit.hit.u as f64;
            let v = rayhit.hit.v as f64;
            [u, v, 1.0 - u - v]
        };

        // Calculate the geometric normal:
        let n = Vec3 {
            x: rayhit.hit.Ng_x as f64,
            y: rayhit.hit.Ng_y as f64,
            z: rayhit.hit.Ng_z as f64,
        };

        let t = rayhit.ray.tfar as f64;

        let wo = Vec3 {
            x: -(rayhit.ray.dir_x as f64),
            y: -(rayhit.ray.dir_y as f64),
            z: -(rayhit.ray.dir_z as f64),
        };

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
            unsafe { self.uvs(mesh) }
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
            let nrms = unsafe { self.nrm(mesh) };
            let shading_n = nrms[0].scale(b[0]) + nrms[1].scale(b[1]) + nrms[2].scale(b[2]);
            let dn02 = nrms[0] - nrms[2];
            let dn12 = nrms[1] - nrms[2];
            let dndu = (dn02.scale(duv12[1]) - dn12.scale(duv02[1])).scale(inv_det);
            let dndv = (dn02.scale(-duv12[0]) + dn12.scale(duv02[0])).scale(inv_det);
            let n = util::align(shading_n, n); // If we have shading normals, let is decide orientation
            (n, shading_n, dndu, dndv)
        };

        // Calculate tangent shading informatin:
        let shading_dpdu = if mesh.tan.is_empty() {
            dpdu.normalize()
        } else {
            let tans = unsafe { self.tan(mesh) };
            tans[0].scale(b[0]) + tans[1].scale(b[1]) + tans[2].scale(b[2])
        };

        let shading_dpdv = shading_n.cross(shading_dpdu).normalize();
        let shading_dpdu = shading_dpdv.cross(shading_n);

        Interaction {
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
        unsafe {
            [
                mesh.pos.get_unchecked(self.indices[0] as usize).to_f64(),
                mesh.pos.get_unchecked(self.indices[1] as usize).to_f64(),
                mesh.pos.get_unchecked(self.indices[2] as usize).to_f64(),
            ]
        }
    }

    pub unsafe fn nrm(self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        [
            mesh.nrm.get_unchecked(self.indices[0] as usize).to_f64(),
            mesh.nrm.get_unchecked(self.indices[1] as usize).to_f64(),
            mesh.nrm.get_unchecked(self.indices[2] as usize).to_f64(),
        ]
    }

    pub unsafe fn tan(self, mesh: &Mesh) -> [Vec3<f64>; 3] {
        [
            mesh.tan.get_unchecked(self.indices[0] as usize).to_f64(),
            mesh.tan.get_unchecked(self.indices[1] as usize).to_f64(),
            mesh.tan.get_unchecked(self.indices[2] as usize).to_f64(),
        ]
    }

    pub unsafe fn uvs(self, mesh: &Mesh) -> [Vec2<f64>; 3] {
        [
            mesh.uvs.get_unchecked(self.indices[0] as usize).to_f64(),
            mesh.uvs.get_unchecked(self.indices[1] as usize).to_f64(),
            mesh.uvs.get_unchecked(self.indices[2] as usize).to_f64(),
        ]
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,

    pub pos: Vec<Vec3<f32>>,
    pub nrm: Vec<Vec3<f32>>,
    pub tan: Vec<Vec3<f32>>,
    pub uvs: Vec<Vec2<f32>>,

    surface_area: Cell<Option<f64>>,

    rtcgeom: Cell<Option<embree::RTCGeometry>>,
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
            surface_area: Cell::new(None),
            rtcgeom: Cell::new(None),
        }
    }

    // Changes the underlying data by applying the provided transformation
    // to transform the points, tangents and normals.
    pub fn transform(&mut self, transform: Transform) {
        transform.points_f32(&mut self.pos);
        transform.vectors_f32(&mut self.tan);
        transform.normals_f32(&mut self.nrm);
    }

    pub fn surface_area(&self) -> f64 {
        if let Some(sa) = self.surface_area.get() {
            return sa;
        }

        let sa = self
            .triangles
            .iter()
            .fold(0.0, |sa, triangle| sa + triangle.area(self));

        self.surface_area.set(Some(sa));
        sa
    }

    // Disables the mesh (it won't be rendered). Is a noop if it was never committed to an rtcgeom
    pub fn disable_mesh(&self) {
        if let Some(rtcgeom) = self.rtcgeom.get() {
            unsafe {
                embree::rtcDisableGeometry(rtcgeom);
            }
        }
    }

    // Enables the mesh (it'll be rendered)
    pub fn enable_mesh(&self) {
        if let Some(rtcgeom) = self.rtcgeom.get() {
            unsafe {
                embree::rtcEnableGeometry(rtcgeom);
            }
        }
    }

    pub fn get_rtcgeom(&self) -> embree::RTCGeometry {
        if let Some(rtcgeom) = self.rtcgeom.get() {
            rtcgeom
        } else {
            ptr::null_mut()
        }
    }

    // Retrieve the geometry information for a specific mesh:
    pub fn create_rtcgeom(&self, device: &Device) -> SimpleResult<embree::RTCGeometry> {
        // Check if it was already created for this specific mesh:
        if let Some(rtcgeom) = self.rtcgeom.get() {
            return Ok(rtcgeom);
        }

        let rtcgeom = device.new_geometry(embree::RTCGeometryType_RTC_GEOMETRY_TYPE_TRIANGLE)?;

        // Set the buffers:
        unsafe {
            embree::rtcSetSharedGeometryBuffer(
                rtcgeom,
                embree::RTCBufferType_RTC_BUFFER_TYPE_INDEX,
                0,
                embree::RTCFormat_RTC_FORMAT_UINT3,
                self.triangles.as_ptr() as *const raw::c_void,
                0,                                            // offset
                mem::size_of::<Triangle>() as embree::size_t, // stride
                self.triangles.len() as embree::size_t,       // number of elements
            );
        }
        device.error()?;

        unsafe {
            embree::rtcSetSharedGeometryBuffer(
                rtcgeom,
                embree::RTCBufferType_RTC_BUFFER_TYPE_VERTEX,
                0,
                embree::RTCFormat_RTC_FORMAT_FLOAT3,
                self.pos.as_ptr() as *const raw::c_void,
                0,                                             // offset
                mem::size_of::<Vec3<f32>>() as embree::size_t, // stride
                // We subtract 1 because we need extra floats at the end for embree to
                // access the data with simd registers.
                (self.pos.len() - 1) as embree::size_t,
            );
        }
        device.error()?;

        self.rtcgeom.set(Some(rtcgeom));
        Ok(rtcgeom)
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if let Some(rtcgeom) = self.rtcgeom.get() {
            unsafe {
                embree::rtcReleaseGeometry(rtcgeom);
            }
            self.rtcgeom.set(None);
        }
    }
}
