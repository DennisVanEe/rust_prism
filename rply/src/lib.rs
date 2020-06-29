// A very simple crate that uses bindgen to create an interface for
// embree that I can use with prism.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/wrapper.rs"));
