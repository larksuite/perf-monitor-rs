//! Get io usage for current process.
use errno::Errno;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("IoStatsError({code}):{msg}")]
pub struct IoStatsError {
    pub code: i32,
    pub msg: String,
}

impl From<Errno> for IoStatsError {
    fn from(e: Errno) -> Self {
        Self {
            code: e.into(),
            msg: e.to_string(),
        }
    }
}

impl From<std::io::Error> for IoStatsError {
    fn from(e: std::io::Error) -> Self {
        Self {
            code: e.kind() as i32,
            msg: e.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for IoStatsError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self {
            code: 0,
            msg: e.to_string(),
        }
    }
}
/// A struct represents io status.
#[derive(Debug, Clone, Default)]
pub struct IoStats {
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
pub fn get_process_io_stats() -> Result<IoStats, IoStatsError> {
    get_process_io_stats_impl()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_process_io_stats_impl() -> Result<IoStats, IoStatsError> {
    use std::{
        io::{BufRead, BufReader},
        str::FromStr,
    };
    let mut io_stats = IoStats::default();
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
fn get_process_io_stats_impl() -> Result<IoStats, IoStatsError> {
    use winapi::um::{
        processthreadsapi::GetCurrentProcess, winbase::GetProcessIoCounters, winnt::IO_COUNTERS,
    };
    let mut io_counters = IO_COUNTERS {
        ReadOperationCount: 0,
        WriteOperationCount: 0,
        OtherOperationCount: 0,
        ReadTransferCount: 0,
        WriteTransferCount: 0,
        OtherTransferCount: 0,
    };
    let ret = unsafe {
        // If the function succeeds, the return value is nonzero.
        // If the function fails, the return value is zero.
        // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getprocessiocounters
        GetProcessIoCounters(GetCurrentProcess(), &mut io_counters)
    };
    if ret != 0 {
        Ok(IoStats {
            read_count: io_counters.ReadOperationCount,
            write_count: io_counters.WriteOperationCount,
            read_bytes: io_counters.ReadTransferCount,
            write_bytes: io_counters.WriteTransferCount,
        })
    } else {
        Err(errno::errno().into())
    }
}

#[cfg(target_os = "macos")]
fn get_process_io_stats_impl() -> Result<IoStats, IoStatsError> {
    use std::{ffi::c_void, os::raw::c_int};

    use crate::bindings::rusage_info_v2 as RUsageInfoV2;

    #[link(name = "proc", kind = "dylib")]
    extern "C" {
        // Return resource usage information for the given pid, which can be a live
        // process or a zombie.
        //
        // Returns 0 on success; or -1 on failure, with errno set to indicate the
        // specific error.
        fn proc_pid_rusage(pid: c_int, flavor: c_int, buffer: *mut c_void) -> c_int;
    }

    let mut rusage_info = RUsageInfoV2::default();
    let ret_code = unsafe {
        proc_pid_rusage(
            std::process::id() as c_int,
            2,
            (&mut rusage_info as *mut RUsageInfoV2).cast::<c_void>(),
        )
    };
    if ret_code == 0 {
        Ok(IoStats {
            read_bytes: rusage_info.ri_diskio_bytesread,
            write_bytes: rusage_info.ri_diskio_byteswritten,
            ..Default::default()
        })
    } else {
        Err(errno::errno().into())
    }
}
