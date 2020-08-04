mod embree_impl;

use lazy_static::lazy_static;
use pmath::numbers::Float;
use pmath::ray::Ray;
use pmath::vector::{Vec2, Vec3};
use std::mem::MaybeUninit;
use std::os::raw;
use std::ptr;

//
// Pointer Types
//

const NULL_USIZE: usize = 0;

//
// Static Device

// Because we only ever have one device (which is initialized by .embree3),
// we can just create this device lazily.
lazy_static! {
    static ref DEVICE: usize = {
        let ptr = unsafe { embree_impl::rtcNewDevice(ptr::null()) } as usize;
        check_device_error_impl(ptr);
        ptr
    };
}

fn get_device_raw() -> embree_impl::RTCDevice {
    *DEVICE as embree_impl::RTCDevice
}

/// Thin wrapper for the embree type: `RTCGeometry`.
#[derive(Copy, Clone, Debug)]
pub struct Geometry {
    data: usize,
}

impl Geometry {
    pub fn new_null() -> Self {
        Self { data: NULL_USIZE }
    }

    pub fn is_null(self) -> bool {
        self.data == NULL_USIZE
    }

    fn from_raw(ptr: embree_impl::RTCGeometry) -> Self {
        Geometry { data: ptr as usize }
    }

    fn get_raw(self) -> embree_impl::RTCGeometry {
        self.data as embree_impl::RTCGeometry
    }
}

/// Thin wrapper for the embree type: `RTCScene`.
#[derive(Clone, Copy, Debug)]
pub struct Scene {
    data: usize,
}

impl Scene {
    pub fn new_null() -> Self {
        Self { data: NULL_USIZE }
    }

    pub fn is_null(self) -> bool {
        self.data == NULL_USIZE
    }

    fn from_raw(ptr: embree_impl::RTCScene) -> Self {
        Scene { data: ptr as usize }
    }

    fn get_raw(self) -> embree_impl::RTCScene {
        self.data as embree_impl::RTCScene
    }
}

//
// Enumerated Types
//

#[derive(Copy, Clone, Debug)]
pub enum IntersectContextFlags {
    // None: is equivelant with incoherent...
    Incoherent =
        embree_impl::RTCIntersectContextFlags_RTC_INTERSECT_CONTEXT_FLAG_INCOHERENT as isize,
    Coherent = embree_impl::RTCIntersectContextFlags_RTC_INTERSECT_CONTEXT_FLAG_COHERENT as isize,
}

impl IntersectContextFlags {
    // fn get_raw(self) -> embree_impl::RTCIntersectContextFlags {
    //     self as embree_impl::RTCIntersectContextFlags
    // }
}

/// Thin wrapper for the embree type: `RTCBuildQuality`.
#[derive(Copy, Clone, Debug)]
pub enum BuildQuality {
    Low = embree_impl::RTCBuildQuality_RTC_BUILD_QUALITY_LOW as isize,
    Medium = embree_impl::RTCBuildQuality_RTC_BUILD_QUALITY_MEDIUM as isize,
    High = embree_impl::RTCBuildQuality_RTC_BUILD_QUALITY_HIGH as isize,
}

impl BuildQuality {
    fn get_raw(self) -> embree_impl::RTCBuildQuality {
        self as embree_impl::RTCBuildQuality
    }
}

/// Thin wrapper for the embree type: `RTCSceneFlags`.
#[derive(Copy, Clone, Debug)]
pub enum SceneFlags {
    Compact = embree_impl::RTCSceneFlags_RTC_SCENE_FLAG_COMPACT as isize,
    Robust = embree_impl::RTCSceneFlags_RTC_SCENE_FLAG_ROBUST as isize,
}

impl SceneFlags {
    fn get_raw(self) -> embree_impl::RTCSceneFlags {
        self as embree_impl::RTCSceneFlags
    }
}

/// Thin wrapper for the embree type: `RTCGeometryType`.
#[derive(Copy, Clone, Debug)]
pub enum GeometryType {
    Triangle = embree_impl::RTCGeometryType_RTC_GEOMETRY_TYPE_TRIANGLE as isize,
    Instance = embree_impl::RTCGeometryType_RTC_GEOMETRY_TYPE_INSTANCE as isize,
    SpherePoint = embree_impl::RTCGeometryType_RTC_GEOMETRY_TYPE_SPHERE_POINT as isize,
}

impl GeometryType {
    fn get_raw(self) -> embree_impl::RTCGeometryType {
        self as embree_impl::RTCGeometryType
    }
}

/// Thin wrapper for the embree type: `RTCBufferType`.
#[derive(Copy, Clone, Debug)]
pub enum BufferType {
    Index = embree_impl::RTCBufferType_RTC_BUFFER_TYPE_INDEX as isize,
    Vertex = embree_impl::RTCBufferType_RTC_BUFFER_TYPE_VERTEX as isize,
}

impl BufferType {
    fn get_raw(self) -> embree_impl::RTCBufferType {
        self as embree_impl::RTCBufferType
    }
}

/// Thin wrapper for the embree type: `RTCFormat`.
#[derive(Copy, Clone, Debug)]
pub enum Format {
    Uint3 = embree_impl::RTCFormat_RTC_FORMAT_UINT3 as isize,
    Float3 = embree_impl::RTCFormat_RTC_FORMAT_FLOAT3 as isize,
    Float4 = embree_impl::RTCFormat_RTC_FORMAT_FLOAT4 as isize,
    Float3x4RowMajor = embree_impl::RTCFormat_RTC_FORMAT_FLOAT3X4_ROW_MAJOR as isize,
}

impl Format {
    fn get_raw(self) -> embree_impl::RTCFormat {
        self as embree_impl::RTCFormat
    }
}

//
// Functions
//

/// Wrapper for `rtcNewScene`.
pub fn new_scene() -> Scene {
    let ptr = unsafe { embree_impl::rtcNewScene(get_device_raw()) };
    check_device_error();
    Scene::from_raw(ptr)
}

/// Wrapper for `rtcNewGeometry`.
pub fn new_geometry(geom_type: GeometryType) -> Geometry {
    let ptr = unsafe { embree_impl::rtcNewGeometry(get_device_raw(), geom_type.get_raw()) };
    check_device_error();
    Geometry::from_raw(ptr)
}

/// Wrapper for `rtcNewGeometry`.
pub fn release_geometry(geometry: Geometry) {
    unsafe {
        embree_impl::rtcReleaseGeometry(geometry.get_raw());
    }
    check_device_error();
}

/// Wrapper for `rtcSetSharedGeometryBuffer`.
pub fn set_shared_geometry_buffer<T>(
    geometry: Geometry,
    btype: BufferType,
    slot: u32,
    format: Format,
    ptr: *const T, // Byte slice
    byte_offset: usize,
    byte_stride: usize,
    item_count: usize,
) {
    unsafe {
        embree_impl::rtcSetSharedGeometryBuffer(
            geometry.get_raw(),
            btype.get_raw(),
            slot,
            format.get_raw(),
            ptr as *const raw::c_void,
            byte_offset as embree_impl::size_t,
            byte_stride as embree_impl::size_t,
            item_count as embree_impl::size_t,
        );
    }
    check_device_error();
}

/// Wrapper for `rtcAttachGeometryByID`.
pub fn attach_geometry_by_id(scene: Scene, geometry: Geometry, geom_id: u32) {
    unsafe {
        embree_impl::rtcAttachGeometryByID(scene.get_raw(), geometry.get_raw(), geom_id);
    }
    check_device_error();
}

/// Wrapper for `rtcCommitGeometry`.
pub fn commit_geometry(geometry: Geometry) {
    unsafe {
        embree_impl::rtcCommitGeometry(geometry.get_raw());
    }
    check_device_error();
}

/// Wrapper for `rtcCommitScene`.
pub fn commit_scene(scene: Scene) {
    unsafe {
        embree_impl::rtcCommitScene(scene.get_raw());
    }
    check_device_error();
}

/// Wrapper for `rtcSetGeometryInstancedScene`.
pub fn set_geometry_instance_scene(geometry: Geometry, scene: Scene) {
    unsafe {
        embree_impl::rtcSetGeometryInstancedScene(geometry.get_raw(), scene.get_raw());
    }
    check_device_error();
}

/// Wrapper for `rtcSetGeometryTimeStepCount`.
pub fn set_geometry_timestep_count(geometry: Geometry, count: u32) {
    unsafe {
        embree_impl::rtcSetGeometryTimeStepCount(geometry.get_raw(), count);
    }
    check_device_error();
}

/// Wrapper for `rtcSetGeometryTransform`
pub fn set_geometry_transform<T>(geometry: Geometry, timestep: u32, format: Format, xfm: *const T) {
    unsafe {
        embree_impl::rtcSetGeometryTransform(
            geometry.get_raw(),
            timestep as raw::c_uint,
            format.get_raw(),
            xfm as *const raw::c_void,
        );
    }
    check_device_error();
}

/// Wrapper for `rtcSetSceneFlags`.
pub fn set_scene_flags(scene: Scene, flags: SceneFlags) {
    unsafe {
        embree_impl::rtcSetSceneFlags(scene.get_raw(), flags.get_raw());
    }
    check_device_error();
}

/// Wrapper for `rtcSetSceneBuildQuality`.
pub fn set_scene_build_quality(scene: Scene, quality: BuildQuality) {
    unsafe {
        embree_impl::rtcSetSceneBuildQuality(scene.get_raw(), quality.get_raw());
    }
    check_device_error();
}

/// A "thick" wrapper for `rtcIntersect1`.
pub fn intersect<T: Float>(
    scene: Scene,
    mut ray: Ray<T>,
    mask: u32,
    id: u32,
    flags: u32,
) -> Option<(Ray<T>, Hit<T>)> {
    // Initialize the rayhit information here:
    let mut rayhit = embree_impl::RTCRayHit {
        ray: embree_impl::RTCRay {
            org_x: ray.org.x.to_f32(),
            org_y: ray.org.y.to_f32(),
            org_z: ray.org.z.to_f32(),
            dir_x: ray.dir.x.to_f32(),
            dir_y: ray.dir.y.to_f32(),
            dir_z: ray.dir.z.to_f32(),
            time: ray.time.to_f32(),
            tnear: ray.t_near.to_f32(),
            tfar: ray.t_far.to_f32(),
            mask,
            id,
            flags,
        },
        hit: unsafe { MaybeUninit::uninit().assume_init() },
    };
    rayhit.hit.geomID = embree_impl::RTC_INVALID_GEOMETRY_ID;

    // Initialize the context:
    let mut context: embree_impl::RTCIntersectContext =
        unsafe { MaybeUninit::uninit().assume_init() };
    embree_impl::rtcInitIntersectContext(&mut context);

    // Perform the intersection:
    unsafe {
        embree_impl::rtcIntersect1(
            scene.get_raw(),
            &mut context as *mut embree_impl::RTCIntersectContext,
            &mut rayhit as *mut embree_impl::RTCRayHit,
        );
    };

    // Check if we had an intersection.
    if rayhit.hit.geomID == embree_impl::RTC_INVALID_GEOMETRY_ID {
        None
    } else {
        ray.t_far = T::from_f32(rayhit.ray.tfar);
        Some((
            ray,
            Hit {
                ng: Vec3 {
                    x: T::from_f32(rayhit.hit.Ng_x),
                    y: T::from_f32(rayhit.hit.Ng_y),
                    z: T::from_f32(rayhit.hit.Ng_z),
                },

                uv: Vec2 {
                    x: T::from_f32(rayhit.hit.u),
                    y: T::from_f32(rayhit.hit.v),
                },

                prim_id: rayhit.hit.primID,
                geom_id: rayhit.hit.geomID,
                inst_id: rayhit.hit.instID,
            },
        ))
    }
}

/// A "thick" wrapper for `rtcOccluded1`.
pub fn occluded<T: Float>(scene: Scene, ray: Ray<T>, mask: u32, id: u32, flags: u32) -> bool {
    // Initialize the rayhit information here:
    let mut embree_ray = embree_impl::RTCRay {
        org_x: ray.org.x.to_f32(),
        org_y: ray.org.y.to_f32(),
        org_z: ray.org.z.to_f32(),
        dir_x: ray.dir.x.to_f32(),
        dir_y: ray.dir.y.to_f32(),
        dir_z: ray.dir.z.to_f32(),
        time: ray.time.to_f32(),
        tnear: ray.t_near.to_f32(),
        tfar: ray.t_far.to_f32(),
        mask,
        id,
        flags,
    };

    // Initialize the context:
    let mut context: embree_impl::RTCIntersectContext =
        unsafe { MaybeUninit::uninit().assume_init() };
    embree_impl::rtcInitIntersectContext(&mut context);

    // Perform the intersection.
    unsafe {
        embree_impl::rtcOccluded1(
            scene.get_raw(),
            &mut context as *mut embree_impl::RTCIntersectContext,
            &mut embree_ray as *mut embree_impl::RTCRay,
        );
    };

    // Check for an intersection.
    embree_ray.tfar == f32::NEG_INFINITY
}

/// Wrapper for `rtcGetDeviceError`. Returns Ok if no error is present, returns error string otherwise.
fn check_device_error() {
    check_device_error_impl(*DEVICE);
}

/// Wrapper for `rtcGetDeviceError`. Returns Ok if no error is present, returns error string otherwise.
fn check_device_error_impl(device: usize) {
    let code = unsafe { embree_impl::rtcGetDeviceError(device as embree_impl::RTCDevice) };
    if code == embree_impl::RTCError_RTC_ERROR_NONE {
        return;
    }

    let err_str = match code {
        embree_impl::RTCError_RTC_ERROR_NONE => "RTC_ERROR_NONE",
        embree_impl::RTCError_RTC_ERROR_INVALID_ARGUMENT => "RTC_ERROR_INVALID_ARGUMENT",
        embree_impl::RTCError_RTC_ERROR_INVALID_OPERATION => "RTC_ERROR_INVALID_OPERATION",
        embree_impl::RTCError_RTC_ERROR_OUT_OF_MEMORY => "RTC_ERROR_OUT_OF_MEMORY",
        embree_impl::RTCError_RTC_ERROR_UNSUPPORTED_CPU => "RTC_ERROR_UNSUPPORTED_CPU",
        embree_impl::RTCError_RTC_ERROR_CANCELLED => "RTC_ERROR_CANCELLED",
        /* embree::RTCError_RTC_ERROR_UNKNOWN*/ _ => "RTC_ERROR_UNKNOWN",
    };

    panic!("{}", err_str);
}

//
// Structures
//

pub const INVALID_GEOM_ID: u32 = embree_impl::RTC_INVALID_GEOMETRY_ID;

/// Represents `RTCHit`.
#[derive(Clone, Copy, Debug)]
pub struct Hit<T: Float> {
    pub ng: Vec3<T>,
    pub uv: Vec2<T>,
    pub prim_id: u32,
    pub geom_id: u32,
    pub inst_id: [u32; 1],
}
