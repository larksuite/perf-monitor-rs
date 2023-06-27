//! Get io usage for current process.
use thiserror::Error;

#[derive(Error, Debug)]
#[error("IOStatsError({code}):{msg}")]
pub struct IOStatsError {
    pub code: i32,
    pub msg: String,
}

impl From<std::io::Error> for IOStatsError {
    fn from(e: std::io::Error) -> Self {
        Self {
            code: e.kind() as i32,
            msg: e.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for IOStatsError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self {
            code: 0,
            msg: e.to_string(),
        }
    }
}
/// A struct represents io status.
#[derive(Debug, Clone, Default)]
pub struct IOStats {
    /// (linux & windows)  the number of read operations performed (cumulative)
    pub read_count: u64,

    /// (linux & windows) the number of write operations performed (cumulative)
    pub write_count: u64,

    /// the number of bytes read (cumulative).
    pub read_bytes: u64,

    /// the number of bytes written (cumulative)
    pub write_bytes: u64,
}
/// Get the io stats of current process. Most platforms are supported.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "windows"
))]
pub fn get_process_io_stats() -> Result<IOStats, IOStatsError> {
    get_process_io_stats_impl()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_process_io_stats_impl() -> Result<IOStats, IOStatsError> {
    use std::{
        io::{BufRead, BufReader},
        str::FromStr,
    };
    let mut io_stats = IOStats::default();
    let reader = BufReader::new(std::fs::File::open("/proc/self/io")?);

    for line in reader.lines() {
        let line = line?;
        let mut s = line.split_whitespace();
        if let (Some(field), Some(value)) = (s.next(), s.next()) {
            match field {
                "syscr:" => io_stats.read_count = u64::from_str(value)?,
                "syscw:" => io_stats.write_count = u64::from_str(value)?,
                "read_bytes:" => io_stats.read_bytes = u64::from_str(value)?,
                "write_bytes:" => io_stats.write_bytes = u64::from_str(value)?,
                _ => continue,
            }
        }
    }

    Ok(io_stats)
}

#[cfg(target_os = "windows")]
fn get_process_io_stats_impl() -> Result<IOStats, IOStatsError> {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::GetProcessIoCounters;
    use windows_sys::Win32::System::Threading::IO_COUNTERS;
    let mut io_counters = MaybeUninit::<IO_COUNTERS>::uninit();
    let ret = unsafe {
        // If the function succeeds, the return value is nonzero.
        // If the function fails, the return value is zero.
        // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getprocessiocounters
        GetProcessIoCounters(GetCurrentProcess(), io_counters.as_mut_ptr())
    };
    if ret == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let io_counters = unsafe { io_counters.assume_init() };
    Ok(IOStats {
        read_count: io_counters.ReadOperationCount,
        write_count: io_counters.WriteOperationCount,
        read_bytes: io_counters.ReadTransferCount,
        write_bytes: io_counters.WriteTransferCount,
    })
}

#[cfg(target_os = "macos")]
fn get_process_io_stats_impl() -> Result<IOStats, IOStatsError> {
    use libc::{rusage_info_v2, RUSAGE_INFO_V2};
    use std::{mem::MaybeUninit, os::raw::c_int};

    let mut rusage_info_v2 = MaybeUninit::<rusage_info_v2>::uninit();
    let ret_code = unsafe {
        libc::proc_pid_rusage(
            std::process::id() as c_int,
            RUSAGE_INFO_V2,
            rusage_info_v2.as_mut_ptr() as *mut _,
        )
    };
    if ret_code != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let rusage_info_v2 = unsafe { rusage_info_v2.assume_init() };
    Ok(IOStats {
        read_bytes: rusage_info_v2.ri_diskio_bytesread,
        write_bytes: rusage_info_v2.ri_diskio_byteswritten,
        ..Default::default()
    })
}
