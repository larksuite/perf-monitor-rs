//! Get file descriptor(say handle for windows) numbers for current process.
//!
//! ```
//! use perf_monitor::fd::fd_count_cur;
//!
//! let count = fd_count_cur().unwrap();
//! ```
//!
//! ## Bottom Layer Interface
//!
//! - Windows: [GetProcessHandleCount](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesshandlecount)
//! - Linux & android: [/proc/{pid}/fd](https://man7.org/linux/man-pages/man5/proc.5.html)
//! - MacOS: [/dev/fd](https://www.informit.com/articles/article.aspx?p=99706&seqNum=15)
//! - iOS Unfortunately there is no api to retrieve the fd count of the process for iOS.
//! Following links contains a available method, but it's complicated and
//! inefficient. <https://stackoverflow.com/questions/4083608/on-ios-iphone-too-many-open-files-need-to-list-open-files-like-lsof>
//!
//! ## Other Process
//!
//! For windows, linux and android(maybe), it is possible to get fd number of other process.
//! However we didn't re-export these function because macos and ios is not supported.
//!

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod android_linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
use android_linux as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
use ios as platform;

use platform::Result;

/// return the fd count of current process
#[inline]
pub fn fd_count_cur() -> Result<usize> {
    let count = platform::fd_count_cur()?;
    Ok(count)
}

#[cfg(any(target_os = "windows", target_os = "android", target_os = "linux"))]
#[inline]
pub fn fd_count_pid(pid: u32) -> Result<usize> {
    let count = platform::fd_count_pid(pid)?;
    Ok(count)
}
