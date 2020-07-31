#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/// This includes the internal part of it:
include!(concat!(env!("OUT_DIR"), "/rtcore.rs"));

// Not sure why, but Bindgen didn't define this either:
pub const RTC_INVALID_GEOMETRY_ID: ::std::os::raw::c_uint = ::std::os::raw::c_uint::max_value(); // code was ((unsigned int)-1)

// Bindgen doesn't work with inlined functions. So I just defined it here.
// I kept the interface as similar as possible so that it's easy to update and
// understand what's happening:
pub unsafe fn rtcInitIntersectContext(context: &mut RTCIntersectContext) {
    context.flags = RTCIntersectContextFlags_RTC_INTERSECT_CONTEXT_FLAG_INCOHERENT;
    context.filter = None; // equiv. to NULL
    for l in 0..RTC_MAX_INSTANCE_LEVEL_COUNT {
        *context.instID.get_unchecked_mut(l as usize) = RTC_INVALID_GEOMETRY_ID;
    }
}
