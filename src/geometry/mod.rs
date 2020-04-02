pub mod mesh;
//pub mod sphere;

use crate::math::vector::{Vec2, Vec3};

use embree;
use simple_error::SimpleResult;

use std::ptr;

// Geometric interaction:
#[derive(Clone, Copy)]
pub struct GeomInteraction {
    pub p: Vec3<f32>,  // intersection point
    pub n: Vec3<f32>,  // geometric normal (of triangle)
    pub wo: Vec3<f32>, // direction of intersection leaving the point

    pub t: f32, // the t value of the intersection of the ray (not time).

    pub uv: Vec2<f32>,   // uv coordinate at the intersection
    pub dpdu: Vec3<f32>, // vectors parallel to the triangle
    pub dpdv: Vec3<f32>,

    pub shading_n: Vec3<f32>,    // the shading normal at this point
    pub shading_dpdu: Vec3<f32>, // the shading dpdu, dpdv at this point
    pub shading_dpdv: Vec3<f32>,
    pub shading_dndu: Vec3<f32>, // the shading dndu, dndv at this point
    pub shading_dndv: Vec3<f32>,
}

/// Basically just RTCHit converted to something more internally friendly:
#[derive(Clone, Copy)]
pub struct RTCInteraction {
    /// Geometric normal (see Embree3 manual):
    pub ng: Vec3<f32>,
    /// uv value (see Embree3 manual):
    pub uv: Vec2<f32>,
    /// primitive id
    pub prim_id: usize,
    // Note that instance ID and geometry ID aren't included.

    /// From RTCRay, specifies the t value of the intersection
    pub tfar: f32,
    /// From RTCRay, specifies the direction of the ray
    pub dir: Vec3<f32>,
}

impl RTCInteraction {
    pub fn new(rtc_hit: &embree::RTCHit, rtc_ray: &embree::RTCRay) -> Self {
        RTCInteraction {
            ng: Vec3 {
                x: rtc_hit.Ng_x as f32,
                y: rtc_hit.Ng_y as f32,
                z: rtc_hit.Ng_z as f32,
            },
            uv: Vec2 {
                x: rtc_hit.u as f32,
                y: rtc_hit.v as f32,
            },
            prim_id: rtc_hit.primID as usize,
            tfar: rtc_ray.tfar,
            dir: Vec3 {
                x: rtc_ray.dir_x,
                y: rtc_ray.dir_y,
                z: rtc_ray.dir_z,
            }
        }
    }
}

// Not to be confused with a group instance, which is a form of instancing exposed to the user.
struct GeometryInstance {
    geometry: Box<dyn Geometry>,
    rtc_geometry: embree::RTCGeometry,
}

/// Manages all of the geometries in a scene. NOT the groups in a scene (that is manages somewhere else).
pub struct GeometryManager {
    geometries: Vec<GeometryInstance>,
}

impl<'a> GeometryManager {
    /// Adds a geometry to the mannager, returning the geometry's ID:
    pub fn add_geometry(&mut self, geometry: Box<dyn Geometry>) -> usize {
        self.geometries.push(GeometryInstance {
            geometry,
            rtc_geometry: ptr::null_mut(),
        });
        self.geometries.len() - 1
    }

    /// Creates all of the geometries:
    pub fn create_rtcgeoms(&mut self, device: embree::RTCDevice) -> SimpleResult<()> {
        for geometry_instance in self.geometries.iter_mut() {
            let rtc_geom = geometry_instance.geometry.as_ref().create_rtcgeom(device)?;
            geometry_instance.rtc_geometry = rtc_geom;
        }
        Ok(())
    }

    /// Given a geometry id returns the embree pointer: `RTCGeometry`.
    pub fn get_rtcgeom(&self, geom_id: usize) -> embree::RTCGeometry {
        // TODO: measure the performance impact of bounds check
        self.geometries[geom_id].rtc_geometry
    }

    /// Given a geometry id and the hit reported by embree, returns a `GeomInteraction`.
    pub fn proc_interaction(&self, hit: RTCInteraction, geom_id: usize) -> GeomInteraction {
        debug_assert!(geom_id < self.geometries.len());
        // TODO: measure the performance impact of bounds check
        self.geometries[geom_id]
            .geometry
            .as_ref()
            .proc_interaction(hit)
    }
}

// The basic geometry trait defines the geometry that PRISM can intersect.

pub trait Geometry {
    /// Creates an embree geometry instance and returns it.
    /// # Arguments
    /// * `id`: the id to give the geometry being created
    /// * `device`: the device the geometry will belong to
    fn create_rtcgeom(&self, device: embree::RTCDevice) -> SimpleResult<embree::RTCGeometry>;
    /// Calculates the surface area of the geometry.
    fn surface_area(&self) -> f32;
    /// Converts a hit provided by embree to a hit used by prism for shading and
    /// whatnot.
    fn proc_interaction(&self, hit: RTCInteraction) -> GeomInteraction;
}
