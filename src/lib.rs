//! This crate provide the ability to retrieve information for profiling.
//!
//!

#![cfg_attr(test, allow(clippy::all, clippy::unwrap_used))]
#![cfg_attr(doc, feature(doc_cfg))]
#![cfg_attr(test, feature(test))]

#[cfg(test)]
extern crate test;

#[allow(warnings)]
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) mod bindings {
    include!(concat!(env!("OUT_DIR"), "/maat_ios_macos_binding.rs"));
}

pub mod cpu;

pub mod mem;

pub mod io;

pub mod fd;
