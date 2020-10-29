//! A wrapper around libmalloc APIs.

use crate::bindings::{
    mach_task_self_, malloc_default_zone, malloc_statistics_t, malloc_zone_t, vm_address_t,
};
use std::{io, str};

/// A Wrapper around `malloc_statistics_t`, originally defined at `libmalloc.h`.
pub type MallocStatistics = malloc_statistics_t;

/// A Wrapper around `malloc_zone_t`, originally defined at `libmalloc.h`.
pub struct MallocZone(*mut malloc_zone_t);

impl MallocZone {
    /// Get the name of this zone.
    pub fn name(&self) -> Result<&str, str::Utf8Error> {
        unsafe { std::ffi::CStr::from_ptr((*self.0).zone_name) }.to_str()
    }
    /// Get the statistics of this zone.
    pub fn statistics(&mut self) -> Option<MallocStatistics> {
        unsafe {
            let mut stats = std::mem::MaybeUninit::<malloc_statistics_t>::zeroed();
            if let Some(f) = (*((*self.0).introspect)).statistics {
                f(self.0, stats.as_mut_ptr());
                Some(stats.assume_init())
            } else {
                None
            }
        }
    }
}
/// Get all malloc zones of current process.
///
/// # Safety
/// CAUTION： `MallocZone`s(*malloc_zone_t) returned by `malloc_get_all_zones`
/// may be destoryed by other threads.
pub unsafe fn malloc_get_all_zones() -> io::Result<Vec<MallocZone>> {
    let mut count: u32 = 0;
    let mut zones: *mut vm_address_t = std::ptr::null_mut();
    let ret = crate::bindings::malloc_get_all_zones(mach_task_self_, None, &mut zones, &mut count);
    if ret != 0 {
        Err(io::Error::from_raw_os_error(ret))
    } else {
        let zones =
            std::slice::from_raw_parts_mut(zones as *mut *mut malloc_zone_t, count as usize)
                .iter()
                .map(|&p| MallocZone(p))
                .collect::<Vec<_>>();
        Ok(zones)
    }
}

/// Get the default malloc zone of current process.
pub fn malloc_get_default_zone() -> MallocZone {
    MallocZone(unsafe { malloc_default_zone() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_malloc_get_all_zones() {
        let zones = unsafe { malloc_get_all_zones().unwrap() };
        assert!(!zones.is_empty());
        let zone_names = zones.iter().map(|z| z.name().unwrap()).collect::<Vec<_>>();
        assert!(zone_names.contains(&"DefaultMallocZone"));
    }

    #[test]
    fn test_malloc_get_default_zone() {
        let zone = malloc_get_default_zone();
        assert_eq!(zone.name().unwrap(), "DefaultMallocZone");
    }

    #[test]
    fn test_malloc_zone_statistics() {
        let zones = unsafe { malloc_get_all_zones() }.unwrap();
        for mut zone in zones {
            let stat = zone.statistics().unwrap();
            assert!(stat.blocks_in_use > 0);
            assert!(stat.size_in_use > 0);
            assert!(stat.max_size_in_use > 0);
            assert!(stat.size_allocated > 0);
        }
    }
}
