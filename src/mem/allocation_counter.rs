use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};

pub const MIN_ALIGN: usize = 16; // module `sys_common` is private. https://doc.rust-lang.org/src/std/sys_common/alloc.rs.html#28

static ALLOCATED: AtomicIsize = AtomicIsize::new(0);
static ENABLE: AtomicBool = AtomicBool::new(false);

/// An allocator tracks inuse allocated bytes.
///
/// The counter is disable by default. Please enable it by `CountingAllocator::enable()` then call `CountingAllocator::get_allocated()` will return the bytes inused.
pub struct CountingAllocator;

impl CountingAllocator {
    /// Get the inuse bytes allocated by rust.
    pub fn get_allocated() -> isize {
        ALLOCATED.load(Ordering::SeqCst)
    }

    /// Check whether the counter is enable.
    pub fn is_enable() -> bool {
        ENABLE.load(Ordering::SeqCst)
    }

    /// Reset the counter.
    pub fn reset() {
        ALLOCATED.store(0, Ordering::SeqCst)
    }

    /// Enable the counter.
    pub fn enable() {
        ENABLE.store(true, Ordering::SeqCst)
    }

    /// Disable the counter.
    pub fn disable() {
        ENABLE.store(false, Ordering::SeqCst)
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() && Self::is_enable() {
            ALLOCATED.fetch_add(layout.size() as isize, Ordering::SeqCst);
        }
        ret
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        if Self::is_enable() {
            ALLOCATED.fetch_sub(layout.size() as isize, Ordering::SeqCst);
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ret: *mut u8 = System.realloc(ptr, layout, new_size);
        if !ret.is_null()
            && Self::is_enable()
            && layout.align() <= MIN_ALIGN
            && layout.align() <= new_size
        {
            ALLOCATED.fetch_add(new_size as isize - layout.size() as isize, Ordering::SeqCst);
        }
        ret
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc_zeroed(layout);
        if !ret.is_null() && Self::is_enable() {
            ALLOCATED.fetch_add(layout.size() as isize, Ordering::SeqCst);
        }
        ret
    }
}
#[cfg(feature = "allocation_counter")]
#[global_allocator]
static _COUNTER: maat::mem::CountingAllocator = maat: mem::CountingAllocator;
