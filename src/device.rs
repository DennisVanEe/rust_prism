use embree;
use simple_error::{bail, SimpleResult};
use std::ffi::CStr;
use std::os::raw;
use std::ptr;

pub struct Device {
    embree_device: embree::RTCDevice,
}

impl Device {
    pub fn new() -> SimpleResult<Self> {
        let device = Device {
            embree_device: unsafe {
                // For now we can just use the default values (empty string)
                embree::rtcNewDevice(CStr::from_bytes_with_nul_unchecked(b"\0").as_ptr())
            },
        };
        device.error()?;

        unsafe {
            embree::rtcSetDeviceErrorFunction(
                device.embree_device,
                Some(device_error_cb),
                ptr::null_mut(),
            );
        }
        // In case there was an error setting the callback function.
        device.error()?;

        Ok(device)
    }

    pub fn error(&self) -> SimpleResult<()> {
        let err_code = unsafe { embree::rtcGetDeviceError(self.embree_device) };
        if let Some(err_msg) = rtcerror_to_str(err_code) {
            bail!(err_msg);
        }
        Ok(())
    }

    // If there was a problem creating the scene, returns None.
    pub fn new_scene(&self) -> SimpleResult<embree::RTCScene> {
        let scene = unsafe { embree::rtcNewScene(self.embree_device) };
        self.error()?;
        Ok(scene)
    }

    pub fn new_geometry(
        &self,
        geom_type: embree::RTCGeometryType,
    ) -> SimpleResult<embree::RTCGeometry> {
        let geometry = unsafe { embree::rtcNewGeometry(self.embree_device, geom_type) };
        self.error()?;
        Ok(geometry)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // Clean up the device stuff:
        unsafe {
            embree::rtcReleaseDevice(self.embree_device);
        }
        self.embree_device = ptr::null_mut();
    }
}

extern "C" fn device_error_cb(
    _: *mut raw::c_void,
    code: embree::RTCError,
    msg: *const raw::c_char,
) {
    let err_code = rtcerror_to_str(code).unwrap();
    let err_msg = unsafe { CStr::from_ptr(msg) };
    eprintln!(
        "Embree Device error with code: {} and message: {}",
        err_code,
        err_msg.to_str().unwrap()
    );
}

// Returns the error code as a string. If it returns None there was no error.
fn rtcerror_to_str(err: embree::RTCError) -> Option<&'static str> {
    match err {
        embree::RTCError_RTC_ERROR_NONE => None,
        embree::RTCError_RTC_ERROR_INVALID_ARGUMENT => Some("RTC_ERROR_INVALID_ARGUMENT"),
        embree::RTCError_RTC_ERROR_INVALID_OPERATION => Some("RTC_ERROR_INVALID_OPERATION"),
        embree::RTCError_RTC_ERROR_OUT_OF_MEMORY => Some("RTC_ERROR_OUT_OF_MEMORY"),
        embree::RTCError_RTC_ERROR_UNSUPPORTED_CPU => Some("RTC_ERROR_UNSUPPORTED_CPU"),
        embree::RTCError_RTC_ERROR_CANCELLED => Some("RTC_ERROR_CANCELLED"),
        embree::RTCError_RTC_ERROR_UNKNOWN => Some("RTC_ERROR_UNKNOWN"),
        _ => Some("unknown error code"),
    }
}
