use embree;
use once_cell::sync::OnceCell;
use std::ffi::CString;
use std::os::raw;

// TODO: make this static and implement send/sync

pub struct Device {
    device: OnceCell<DevicePtr>,
}

impl Device {
    pub const fn new() -> Self {
        Device {
            device: OnceCell::new(),
        }
    }

    pub fn create_device(&self, options: &str) {
        let cstr = CString::new(options.as_bytes()).unwrap();
        let device = unsafe { embree::rtcNewDevice(cstr.as_ptr()) };
        error_panic(device, "Error calling \"rtcNewDevice\"");
        if let Err(_) = self.device.set(DevicePtr::from_raw(device)) {
            panic!("Error setting device: device was already set");
        }
    }

    pub fn new_geometry(&self, geom_type: GeometryType) -> GeometryPtr {
        let device = self.get_device("Error calling \"rtcNewGeometry\"");
        let ptr = unsafe { embree::rtcNewGeometry(device, geom_type.get_raw()) };
        error_panic(device, "Error calling \"rtcNewGeometry\"");
        GeometryPtr::from_raw(ptr)
    }

    pub fn release_geometry(&self, geometry: GeometryPtr) {
        let device = self.get_device("Error calling \"rtcReleaseGeometry\"");
        unsafe {
            embree::rtcReleaseGeometry(geometry.get_raw());
        }
        error_panic(device, "Error calling \"rtcReleaseGeometry\"");
    }

    pub fn set_shared_geometry_buffer(
        &self,
        geometry: GeometryPtr,
        btype: BufferType,
        slot: u32,
        format: Format,
        ptr: *const raw::c_void,
        byte_offset: usize,
        byte_stride: usize,
        item_count: usize,
    ) {
        let device = self.get_device("Error calling \"rtcSetSharedGeometryBuffer\"");
        unsafe {
            embree::rtcSetSharedGeometryBuffer(
                geometry.get_raw(),
                btype.get_raw(),
                slot as raw::c_uint,
                format.get_raw(),
                ptr,
                byte_offset as embree::size_t,
                byte_stride as embree::size_t,
                item_count as embree::size_t,
            );
        }
        error_panic(device, "Error calling \"rtcSetSharedGeometryBuffer\"");
    }

    pub fn new_scene(&self) -> ScenePtr {
        let device = self.get_device("Error calling \"rtcNewScene\"");
        let ptr = unsafe { embree::rtcNewScene(device) };
        error_panic(device, "Error calling \"rtcNewScene\"");
        ScenePtr::from_raw(ptr)
    }

    pub fn attach_geometry_by_id(&self, scene: ScenePtr, geometry: GeometryPtr, geom_id: u32) {
        let device = self.get_device("Error calling \"rtcAttachGeometryByID\"");
        unsafe {
            embree::rtcAttachGeometryByID(
                scene.get_raw(),
                geometry.get_raw(),
                geom_id as raw::c_uint,
            );
        }
        error_panic(device, "Error calling \"rtcAttachGeometryByID\"");
    }

    pub fn commit_geometry(&self, geometry: GeometryPtr) {
        let device = self.get_device("Error calling \"rtcCommitGeometry\"");
        unsafe {
            embree::rtcCommitGeometry(geometry.get_raw());
        }
        error_panic(device, "Error calling \"rtcCommitGeometry\"");
    }

    pub fn commit_scene(&self, scene: ScenePtr) {
        let device = self.get_device("Error calling \"rtcCommitScene\"");
        unsafe {
            embree::rtcCommitScene(scene.get_raw());
        }
        error_panic(device, "Error calling \"rtcCommitScene\"");
    }

    pub fn set_geometry_instance_scene(&self, geometry: GeometryPtr, scene: ScenePtr) {
        let device = self.get_device("Error calling \"rtcSetGeometryInstancedScene\"");
        unsafe {
            embree::rtcSetGeometryInstancedScene(geometry.get_raw(), scene.get_raw());
        }
        error_panic(device, "Error calling \"rtcSetGeometryInstancedScene\"");
    }

    pub fn set_geometry_timestep_count(&self, geometry: GeometryPtr, count: u32) {
        let device = self.get_device("Error calling \"rtcSetGeometryTimeStepCount\"");
        unsafe {
            embree::rtcSetGeometryTimeStepCount(geometry.get_raw(), count as raw::c_uint);
        }
        error_panic(device, "Error calling \"rtcSetGeometryTimeStepCount\"");
    }

    pub fn set_geometry_transform(
        &self,
        geometry: GeometryPtr,
        timestep: u32,
        format: Format,
        xfm: *const raw::c_void,
    ) {
        let device = self.get_device("Error calling \"rtcSetGeometryTransform\"");
        unsafe {
            embree::rtcSetGeometryTransform(
                geometry.get_raw(),
                timestep as raw::c_uint,
                format.get_raw(),
                xfm,
            );
        }
        error_panic(device, "Error calling \"rtcSetGeometryTransform\"");
    }

    pub fn set_scene_flags(&self, scene: ScenePtr, flags: SceneFlags) {
        let device = self.get_device("Error calling \"rtcSetSceneFlags\"");
        unsafe {
            embree::rtcSetSceneFlags(scene.get_raw(), flags.get_raw());
        }
        error_panic(device, "Error calling \"rtcSetSceneFlags\"");
    }

    pub fn set_scene_build_quality(&self, scene: ScenePtr, quality: BuildQuality) {
        let device = self.get_device("Error calling \"rtcSetSceneBuildQuality\"");
        unsafe {
            embree::rtcSetSceneBuildQuality(scene.get_raw(), quality.get_raw());
        }
        error_panic(device, "Error calling \"rtcSetSceneBuildQuality\"");
    }

    /// Gets the current device:
    fn get_device(&self, msg: &str) -> embree::RTCDevice {
        match self.device.get() {
            Some(device) => device.get_raw(),
            _ => panic!("{}: Device was not yet created", msg),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let device = match self.device.get() {
            Some(device) => device.get_raw(),
            _ => return,
        };
        unsafe {
            embree::rtcReleaseDevice(device);
        }
    }
}

pub static DEVICE: Device = Device::new();

//
// Pointer Types:
//

/// Representation of null as a usize (probably just 0).
const NULL_USIZE: usize = 0;

/// Thin wrapper for the embree type: `RTCDevice`.
#[derive(Copy, Clone, Debug)]
pub struct DevicePtr {
    data: usize,
}

impl DevicePtr {
    pub fn new_null() -> Self {
        DevicePtr { data: NULL_USIZE }
    }

    pub fn is_null(self) -> bool {
        self.data == NULL_USIZE
    }

    pub fn from_raw(ptr: embree::RTCDevice) -> Self {
        DevicePtr { data: ptr as usize }
    }

    pub fn get_raw(self) -> embree::RTCDevice {
        self.data as embree::RTCDevice
    }

    pub fn get_value(self) -> usize {
        self.data
    }
}

/// Thin wrapper for the embree type: `RTCGeometry`.
#[derive(Copy, Clone, Debug)]
pub struct GeometryPtr {
    data: usize,
}

impl GeometryPtr {
    pub fn new_null() -> Self {
        GeometryPtr { data: NULL_USIZE }
    }

    pub fn is_null(self) -> bool {
        self.data == NULL_USIZE
    }

    pub fn from_raw(ptr: embree::RTCGeometry) -> Self {
        GeometryPtr { data: ptr as usize }
    }

    pub fn get_raw(self) -> embree::RTCGeometry {
        self.data as embree::RTCGeometry
    }

    pub fn get_value(self) -> usize {
        self.data
    }
}

/// Thin wrapper for the embree type: `RTCScene`.
#[derive(Copy, Clone, Debug)]
pub struct ScenePtr {
    data: usize,
}

impl ScenePtr {
    pub fn new_null() -> Self {
        ScenePtr { data: NULL_USIZE }
    }

    pub fn is_null(self) -> bool {
        self.data == NULL_USIZE
    }

    pub fn from_raw(ptr: embree::RTCScene) -> Self {
        ScenePtr { data: ptr as usize }
    }

    pub fn get_raw(self) -> embree::RTCScene {
        self.data as embree::RTCScene
    }

    pub fn get_value(self) -> usize {
        self.data
    }
}

//
// Ray Types
//

pub struct 

//
// Enum Types
//

// TODO: make SceneFlags a bitflag enum

#[derive(Copy, Clone, Debug)]
pub enum BuildQuality {
    Low = embree::RTCBuildQuality_RTC_BUILD_QUALITY_LOW as isize,
    Medium = embree::RTCBuildQuality_RTC_BUILD_QUALITY_MEDIUM as isize,
    High = embree::RTCBuildQuality_RTC_BUILD_QUALITY_HIGH as isize,
}

impl BuildQuality {
    pub fn get_raw(self) -> embree::RTCBuildQuality {
        self as embree::RTCBuildQuality
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SceneFlags {
    Compact = embree::RTCSceneFlags_RTC_SCENE_FLAG_COMPACT as isize,
    Robust = embree::RTCSceneFlags_RTC_SCENE_FLAG_ROBUST as isize,
}

impl SceneFlags {
    pub fn get_raw(self) -> embree::RTCSceneFlags {
        self as embree::RTCSceneFlags
    }
}

/// Thin wrapper for the embree type: `RTCGeometryType`.
#[derive(Copy, Clone, Debug)]
pub enum GeometryType {
    Triangle = embree::RTCGeometryType_RTC_GEOMETRY_TYPE_TRIANGLE as isize,
    Instance = embree::RTCGeometryType_RTC_GEOMETRY_TYPE_INSTANCE as isize,
}

impl GeometryType {
    pub fn get_raw(self) -> embree::RTCGeometryType {
        self as embree::RTCGeometryType
    }
}

/// Thin wrapper for the embree type: `RTCBufferType`.
#[derive(Copy, Clone, Debug)]
pub enum BufferType {
    Index = embree::RTCBufferType_RTC_BUFFER_TYPE_INDEX as isize,
    Vertex = embree::RTCBufferType_RTC_BUFFER_TYPE_VERTEX as isize,
}

impl BufferType {
    fn get_raw(self) -> embree::RTCBufferType {
        self as embree::RTCBufferType
    }
}

/// Thin wrapper for the embree type: `RTCFormat`.
#[derive(Copy, Clone, Debug)]
pub enum Format {
    Uint3 = embree::RTCFormat_RTC_FORMAT_UINT3 as isize,
    Float3 = embree::RTCFormat_RTC_FORMAT_FLOAT3 as isize,
    Float3x4RowMajor = embree::RTCFormat_RTC_FORMAT_FLOAT3X4_ROW_MAJOR as isize,
}

impl Format {
    pub fn get_raw(self) -> embree::RTCFormat {
        self as embree::RTCFormat
    }
}

//
// Error Stuff and Misc.
//

/// Given an extra msg, checks if the device operations occured correctly.
fn error_panic(device: embree::RTCDevice, msg: &str) {
    let code = unsafe { embree::rtcGetDeviceError(device) };
    if code == embree::RTCError_RTC_ERROR_NONE {
        return;
    }

    let err_str = match code {
        embree::RTCError_RTC_ERROR_NONE => "RTC_ERROR_NONE",
        embree::RTCError_RTC_ERROR_INVALID_ARGUMENT => "RTC_ERROR_INVALID_ARGUMENT",
        embree::RTCError_RTC_ERROR_INVALID_OPERATION => "RTC_ERROR_INVALID_OPERATION",
        embree::RTCError_RTC_ERROR_OUT_OF_MEMORY => "RTC_ERROR_OUT_OF_MEMORY",
        embree::RTCError_RTC_ERROR_UNSUPPORTED_CPU => "RTC_ERROR_UNSUPPORTED_CPU",
        embree::RTCError_RTC_ERROR_CANCELLED => "RTC_ERROR_CANCELLED",
        /* embree::RTCError_RTC_ERROR_UNKNOWN*/ _ => "RTC_ERROR_UNKNOWN",
    };

    panic!("{}: {}", msg, err_str);
}
