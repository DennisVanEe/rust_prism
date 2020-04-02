use super::{GeomInteraction, Geometry, RTCInteraction};
use crate::math::util::{align, coord_system};
use crate::math::vector::{Vec2, Vec3};

use embree;
use simple_error::{bail, SimpleResult};

use std::cell::Cell;
use std::mem;
use std::os::raw;
use std::ptr;

// Some tests to run to make sure that everything is aligned and sized properly.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_vec3_f32_format() {
        assert_eq!(
            mem::size_of::<Vec3<f32>>(),
            mem::size_of::<raw::c_float>() * 3
        );
        assert_eq!(
            mem::align_of::<Vec3<f32>>(),
            mem::align_of::<raw::c_float>()
        );
    }

    #[test]
    fn correct_triangle_format() {
        assert_eq!(
            mem::size_of::<Triangle>(),
            mem::size_of::<raw::c_uint>() * 3
        );
        assert_eq!(mem::align_of::<Triangle>(), mem::align_of::<raw::c_uint>());
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub indices: [u32; 3],
}

impl Triangle {
    // Calculates the surface area of a specific triangle:
    fn area(&self, pos: &[Vec3<f32>]) -> f32 {
        let poss = self.get_vertices(pos);
        let a = poss[1] - poss[0];
        let b = poss[2] - poss[0];
        a.cross(b).length() * 0.5
    }

    fn get_vertices<T: Copy>(&self, data: &[T]) -> [T; 3] {
        unsafe {
            [
                *data.get_unchecked(self.indices[0] as usize),
                *data.get_unchecked(self.indices[1] as usize),
                *data.get_unchecked(self.indices[2] as usize),
            ]
        }
    }
}

// MeshData represents the collection of data used to represent
// the 3D geometry.
#[derive(Clone, Debug)]
pub struct TriMesh {
    pos: Vec<Vec3<f32>>,
    nrm: Vec<Vec3<f32>>,
    tan: Vec<Vec3<f32>>,
    uvs: Vec<Vec2<f32>>,
    indices: Vec<Triangle>,

    surface_area: Cell<Option<f32>>,
}

impl Geometry for TriMesh {
    fn create_rtcgeom(&self, device: embree::RTCDevice) -> SimpleResult<embree::RTCGeometry> {
        let rtcgeom = unsafe {
            embree::rtcNewGeometry(device, embree::RTCGeometryType_RTC_GEOMETRY_TYPE_TRIANGLE)
        };
        // Check if there was an error:
        if ptr::eq(rtcgeom, ptr::null()) {
            // Get the error code:
            let error_code = unsafe {
                embree::rtcGetDeviceError(device)
            };
            bail!("Error creating geometry with code: {}", error_code);
        }

        // Now we need to add up the number of slots for the geometry:
        unsafe {
            embree::rtcSetGeometryVertexAttributeCount(rtcgeom, 2);
        }

        // Attach the vertex buffer:
        let pos_ptr = self.pos.as_ptr();
        unsafe {
            let pos_void_ptr = pos_ptr as *const raw::c_void;
            embree::rtcSetSharedGeometryBuffer(
                rtcgeom,
                embree::RTCBufferType_RTC_BUFFER_TYPE_VERTEX,
                0,
                embree::RTCFormat_RTC_FORMAT_FLOAT3,
                pos_void_ptr,
                0,
                mem::size_of::<Vec3<f32>>() as embree::size_t,
                (self.pos.len() - 1) as embree::size_t, // This was done so that embree can load 4 at a time
            );
        }

        // Attach the index buffer:
        let indices_ptr = self.indices.as_ptr();
        unsafe {
            let indices_void_ptr = indices_ptr as *const raw::c_void;
            embree::rtcSetSharedGeometryBuffer(
                rtcgeom,
                embree::RTCBufferType_RTC_BUFFER_TYPE_INDEX,
                1,
                embree::RTCFormat_RTC_FORMAT_UINT3,
                indices_void_ptr,
                0,
                mem::size_of::<Triangle>() as embree::size_t,
                self.indices.len() as embree::size_t,
            );
        }

        Ok(rtcgeom)
    }

    fn surface_area(&self) -> f32 {
        if let Some(s) = self.surface_area.get() {
            s
        } else {
            let s = self
                .indices
                .iter()
                .fold(0., |area, face| area + face.area(&self.pos));
            self.surface_area.set(Some(s));
            s
        }
    }

    fn proc_interaction(&self, hit: RTCInteraction) -> GeomInteraction {
        // The triangle that the ray had hit:
        let triangle = unsafe { *self.indices.get_unchecked(hit.prim_id) };

        // Calculate the geometric normal:
        let n = hit.ng.normalize();

        // Calculate the partial derivatives of the triangle

        let poss = triangle.get_vertices(self.get_pos());
        let uvs = if self.has_uvs() {
            triangle.get_vertices(self.get_uvs())
        } else {
            [
                Vec2 { x: 0., y: 0. },
                Vec2 { x: 1., y: 0. },
                Vec2 { x: 1., y: 1. },
            ]
        };

        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];
        let dp02 = poss[0] - poss[2];
        let dp12 = poss[1] - poss[2];

        let determinant = duv02.x * duv12.y - duv02.y * duv12.x;
        let (dpdu, dpdv) = if determinant == 0. {
            coord_system(n)
        } else {
            let inv_determinant = 1. / determinant;
            (
                (dp02.scale(duv12.y) - dp12.scale(duv02.y)).scale(inv_determinant),
                (dp02.scale(-duv12.x) + dp12.scale(duv02.x)).scale(inv_determinant),
            )
        };

        // We can extract 3D barycentric coordinates as follows:
        let bs = [1. - hit.uv.x - hit.uv.y, hit.uv.x, hit.uv.y];

        // Calculate the hit point:
        let p = poss[0].scale(bs[0]) + poss[1].scale(bs[1]) + poss[2].scale(bs[2]);
        // Calculate the uv point:
        let uv = uvs[0].scale(bs[0]) + uvs[1].scale(bs[1]) + uvs[2].scale(bs[2]);

        // TODO: add support for texture alpha checking

        // Compute the shading normal and update the geometric normal:
        let (dndu, dndv, n, shading_n) = if self.has_nrm() {
            let norms = triangle.get_vertices(self.get_nrm());
            let shading_n = norms[0].scale(bs[0]) + norms[1].scale(bs[1]) + norms[2].scale(bs[2]);

            // Calculate the dndu, dndv now:
            let dn1 = norms[0] - norms[2];
            let dn2 = norms[1] - norms[2];

            let (dndu, dndv) = if determinant == 0. {
                (Vec3::zero(), Vec3::zero())
            } else {
                let inv_determinant = 1. / determinant;
                (
                    (dn1.scale(duv12.y) - dn2.scale(duv02.y)).scale(inv_determinant),
                    (dn1.scale(-duv12.x) + dn2.scale(duv02.x)).scale(inv_determinant),
                )
            };

            // Make sure the geometric normal points in the same direction as the provided shading normal:
            (dndu, dndv, align(shading_n, n), shading_n)
        } else {
            (Vec3::zero(), Vec3::zero(), n, n)
        };

        // Compute the tangent:
        let ss = if self.has_tan() {
            let tans = triangle.get_vertices(self.get_tan());
            tans[0].scale(bs[0]) + tans[1].scale(bs[1]) + tans[2].scale(bs[2])
        } else {
            dpdu.normalize()
        };

        let ts = shading_n.cross(ss);
        let (ss, ts) = if ts.length2() > 0. {
            let ts = ts.normalize();
            (ts.cross(shading_n), ts)
        } else {
            coord_system(shading_n)
        };

        GeomInteraction {
            p,
            n,
            wo: -(hit.dir.normalize()),
            t: hit.tfar,
            uv,
            dpdu,
            dpdv,
            shading_n,
            shading_dpdu: ss,
            shading_dpdv: ts,
            shading_dndu: dndu,
            shading_dndv: dndv,
        }
    }
}

// Mesh access is done through u32 values to save on storage:
impl TriMesh {
    pub fn new(
        indices: Vec<Triangle>,
        pos: Vec<Vec3<f32>>,
        nrm: Vec<Vec3<f32>>,
        tan: Vec<Vec3<f32>>,
        uvs: Vec<Vec2<f32>>,
    ) -> Self {
        TriMesh {
            pos,
            nrm,
            tan,
            uvs,
            indices,
            surface_area: Cell::new(None),
        }
    }

    pub fn num_vert(&self) -> usize {
        self.pos.len() - 1
    }

    pub fn has_nrm(&self) -> bool {
        !self.nrm.is_empty()
    }

    pub fn has_tan(&self) -> bool {
        !self.tan.is_empty()
    }

    pub fn has_uvs(&self) -> bool {
        !self.uvs.is_empty()
    }

    pub fn get_pos(&self) -> &[Vec3<f32>] {
        unsafe {
            let last_index = self.pos.len() - 1;
            self.pos.get_unchecked(..last_index)
        }
    }

    pub fn get_pos_mut(&mut self) -> &mut [Vec3<f32>] {
        unsafe {
            let last_index = self.pos.len() - 1;
            self.pos.get_unchecked_mut(..last_index)
        }
    }

    pub fn get_nrm(&self) -> &[Vec3<f32>] {
        &self.nrm[..]
    }

    pub fn get_nrm_mut(&mut self) -> &mut [Vec3<f32>] {
        &mut self.nrm[..]
    }

    pub fn get_tan(&self) -> &[Vec3<f32>] {
        &self.tan[..]
    }

    pub fn get_tan_mut(&mut self) -> &mut [Vec3<f32>] {
        &mut self.tan[..]
    }

    pub fn get_uvs(&self) -> &[Vec2<f32>] {
        &self.uvs[..]
    }

    pub fn get_uvs_mut(&mut self) -> &mut [Vec2<f32>] {
        &mut self.uvs[..]
    }
}
