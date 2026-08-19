//! Raw FFI bindings to the GoldSrc engine (generated via bindgen) plus a
//! small set of hand-written helpers in [`ffi`].

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::type_complexity)]

/// Hand-written engine function pointer helpers.
pub mod ffi;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
