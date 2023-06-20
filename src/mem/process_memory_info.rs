use std::io::{Error, Result};

/// Process Memory Info returned by `get_process_memory_info`
#[derive(Clone, Default)]
pub struct ProcessMemoryInfo {
    /// this is the non-swapped physical memory a process has used.
    /// On UNIX it matches `top`'s RES column).
    ///
    /// On Windows this is an alias for wset field and it matches "Mem Usage"
    /// column of taskmgr.exe.
    pub resident_set_size: u64,
    pub resident_set_size_peak: u64,

    /// this is the total amount of virtual memory used by the process.
    /// On UNIX it matches `top`'s VIRT column.
    ///
    /// On Windows this is an alias for pagefile field and it matches "Mem
    /// Usage" "VM Size" column of taskmgr.exe.
    pub virtual_memory_size: u64,

    ///  This is the sum of:
    ///
    ///    + (internal - alternate_accounting)
    ///
    ///    + (internal_compressed - alternate_accounting_compressed)
    ///
    ///    + iokit_mapped
    ///
    ///    + purgeable_nonvolatile
    ///
    ///    + purgeable_nonvolatile_compressed
    ///
    ///    + page_table
    ///
    /// details: <https://github.com/apple/darwin-xnu/blob/master/osfmk/kern/task.c>
    #[cfg(target_os = "macos")]
    pub phys_footprint: u64,

    #[cfg(target_os = "macos")]
    pub compressed: u64,
}

#[cfg(target_os = "windows")]
fn get_process_memory_info_impl() -> Result<ProcessMemoryInfo> {
    use winapi::um::{
        processthreadsapi::GetCurrentProcess,
        psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
    };
    let mut process_memory_counters = PROCESS_MEMORY_COUNTERS {
        cb: 0,
        PageFaultCount: 0,
        PeakWorkingSetSize: 0,
        WorkingSetSize: 0,
        QuotaPeakPagedPoolUsage: 0,
        QuotaPagedPoolUsage: 0,
        QuotaPeakNonPagedPoolUsage: 0,
        QuotaNonPagedPoolUsage: 0,
        PagefileUsage: 0,
        PeakPagefileUsage: 0,
    };
    let ret = unsafe {
        // If the function succeeds, the return value is nonzero.
        // If the function fails, the return value is zero.
        // https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getprocessmemoryinfo
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut process_memory_counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    if ret == 0 {
        return Err(Error::last_os_error());
    }
    Ok(ProcessMemoryInfo {
        resident_set_size: process_memory_counters.WorkingSetSize as u64,
        resident_set_size_peak: process_memory_counters.PeakWorkingSetSize as u64,
        virtual_memory_size: process_memory_counters.PagefileUsage as u64,
    })
}

#[cfg(target_os = "macos")]
fn get_process_memory_info_impl() -> Result<ProcessMemoryInfo> {
    use crate::bindings::task_vm_info as TaskVMInfo;
    use mach::{
        kern_return::KERN_SUCCESS, message::mach_msg_type_number_t, task::task_info,
        task_info::TASK_VM_INFO, traps::mach_task_self, vm_types::natural_t,
    };

    let mut task_vm_info = TaskVMInfo::default();
    let task_vm_info_ptr = (&mut task_vm_info) as *mut TaskVMInfo;

    // https://github.com/apple/darwin-xnu/blob/master/osfmk/mach/task_info.h line 396
    // #define TASK_VM_INFO_COUNT	((mach_msg_type_number_t) \
    // (sizeof (task_vm_info_data_t) / sizeof (natural_t)))
    let mut task_info_cnt: mach_msg_type_number_t = (std::mem::size_of::<TaskVMInfo>()
        / std::mem::size_of::<natural_t>())
        as mach_msg_type_number_t;

    let kern_ret = unsafe {
        task_info(
            mach_task_self(),
            TASK_VM_INFO,
            task_vm_info_ptr.cast::<i32>(),
            &mut task_info_cnt,
        )
    };
    if kern_ret != KERN_SUCCESS {
        // see https://docs.rs/mach/0.2.3/mach/kern_return/index.html for more details
        return Err(Error::new(
            std::io::ErrorKind::Other,
            format!("DARWIN_KERN_RET_CODE:{}", kern_ret),
        ));
    }
    Ok(ProcessMemoryInfo {
        resident_set_size: task_vm_info.resident_size,
        resident_set_size_peak: task_vm_info.resident_size_peak,
        virtual_memory_size: task_vm_info.virtual_size,
        phys_footprint: task_vm_info.phys_footprint,
        compressed: task_vm_info.compressed,
    })
}

pub fn get_process_memory_info() -> Result<ProcessMemoryInfo> {
    get_process_memory_info_impl()
}
