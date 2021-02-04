//! This sub-mod provides some facilities about memory performance profiling.
//! # Memory usage of current process
//! There's a platform-related function called `get_process_memory_info` available on MacOS and Windows.
//! # Memory usage of ALL Rust allocations
//! We provide a `CountingAllocator` that wraps the system allocator but tracks the bytes used by rust allocations.
//! This crate DOES NOT replace the global allocator by default. You need to make it as a `global_allocator` or enable the `allocation_counter` feature.
//! ```ignore
//! #[global_allocator]
//! static _COUNTER: maat::mem::CountingAllocator = maat:mem::CountingAllocator;
//! ```

mod allocation_counter;

pub use allocation_counter::CountingAllocator;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod process_memory_info;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use process_memory_info::{get_process_memory_info, ProcessMemoryInfo, ProcessMemoryInfoError};

#[cfg(target_os = "macos")]
#[cfg_attr(doc, doc(cfg(macos)))]
pub mod apple;
