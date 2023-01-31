//! A wrapper around `mach_vm_region` API.

use mach::{
    kern_return::KERN_SUCCESS,
    mach_types::vm_task_entry_t,
    message::mach_msg_type_number_t,
    port::mach_port_t,
    traps::mach_task_self,
    vm_page_size::vm_page_size,
    vm_region::{vm_region_extended_info_data_t, vm_region_info_t, VM_REGION_EXTENDED_INFO},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};
use std::{marker::PhantomData, mem};

// `vm_region_info_t`'s `user_tag`, originally defined at osfmk/mach/vm_statistics.h
#[derive(Debug)]
pub enum VMRegionKind {
    Malloc,
    MallocSmall,
    MallocLarge,
    MallocHuge,
    Sbrk,
    Realloc,
    MallocTiny,
    MallocLargeReusable,
    MallocLargeReused,
    Stack,
    MallocNano,
    Dylib,
    Dyld,
    DyldMalloc,
    Tag(u32),
}

impl From<u32> for VMRegionKind {
    fn from(user_tag: u32) -> Self {
        match user_tag {
            1 => VMRegionKind::Malloc,
            2 => VMRegionKind::MallocSmall,
            3 => VMRegionKind::MallocLarge,
            4 => VMRegionKind::MallocHuge,
            5 => VMRegionKind::Sbrk,
            6 => VMRegionKind::Realloc,
            7 => VMRegionKind::MallocTiny,
            8 => VMRegionKind::MallocLargeReusable,
            9 => VMRegionKind::MallocLargeReused,
            11 => VMRegionKind::MallocNano,
            30 => VMRegionKind::Stack,
            33 => VMRegionKind::Dylib,
            60 => VMRegionKind::Dyld,
            61 => VMRegionKind::DyldMalloc,
            tag => VMRegionKind::Tag(tag),
        }
    }
}
// A wrapper around `vm_region_extended_info` with addr, size ... props.
#[derive(Debug)]
pub struct VMRegion {
    addr: mach_vm_address_t,
    size: mach_vm_size_t,
    info: mach::vm_region::vm_region_extended_info,
}

impl VMRegion {
    pub fn kind(&self) -> VMRegionKind {
        VMRegionKind::from(self.info.user_tag)
    }

    pub fn dirty_bytes(&self) -> usize {
        self.info.pages_dirtied as usize * unsafe { vm_page_size }
    }

    pub fn swapped_bytes(&self) -> usize {
        self.info.pages_swapped_out as usize * unsafe { vm_page_size }
    }

    pub fn resident_bytes(&self) -> usize {
        self.info.pages_dirtied as usize * unsafe { vm_page_size }
    }

    fn end_addr(&self) -> mach_vm_address_t {
        self.addr + self.size as mach_vm_address_t
    }
}
// An iter over VMRegions.
pub struct VMRegionIter {
    task: vm_task_entry_t,
    addr: mach_vm_address_t,
    _mark: PhantomData<*const ()>, // make it !Sync & !Send
}

impl Default for VMRegionIter {
    fn default() -> Self {
        Self {
            task: unsafe { mach_task_self() } as vm_task_entry_t,
            addr: 1,
            _mark: PhantomData,
        }
    }
}

impl Iterator for VMRegionIter {
    type Item = VMRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let mut count = mem::size_of::<vm_region_extended_info_data_t>() as mach_msg_type_number_t;
        let mut object_name: mach_port_t = 0;
        let mut size = unsafe { mem::zeroed::<mach_vm_size_t>() };
        let mut info = unsafe { mem::zeroed::<vm_region_extended_info_data_t>() };
        let result = unsafe {
            mach::vm::mach_vm_region(
                self.task,
                &mut self.addr,
                &mut size,
                VM_REGION_EXTENDED_INFO,
                &mut info as *mut vm_region_extended_info_data_t as vm_region_info_t,
                &mut count,
                &mut object_name,
            )
        };
        if result != KERN_SUCCESS {
            None
        } else {
            let region = VMRegion {
                addr: self.addr,
                size,
                info,
            };
            self.addr = region.end_addr();
            Some(region)
        }
    }
}
