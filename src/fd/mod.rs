//! Get file descriptor(say handle for windows) numbers for current process.
//!
//! ```
//! use maat::fd::fd_count_cur;
//!
//! let count = fd_count_cur().unwrap();
//! ```
//!
//! ## Bottom Layer Interface
//!
//! - windows: [GetProcessHandleCount](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesshandlecount)
//! - linux & android: [/proc/<pid>/fd](https://man7.org/linux/man-pages/man5/proc.5.html)
//! - macos: [/dev/fd](https://www.informit.com/articles/article.aspx?p=99706&seqNum=15)
//! - ios: Unfortunately there is no api to retrieve the fd count of the process for IOS.
//! Following links contains a available method, but it's complicated and
//! inefficient. <https://stackoverflow.com/questions/4083608/on-ios-iphone-too-many-open-files-need-to-list-open-files-like-lsof>
//!
//! ## Other Process
//!
//! For windows, linux and android(maybe), it is possible to get fd number of other process.
//! However we didn't re-export these function because macos and ios is not supported.
//!

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod android_linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
use android_linux as platform;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "ios")]
pub mod ios;
#[cfg(target_os = "ios")]
use ios as platform;

use platform::Result;

/// return the fd count of current process
#[inline]
pub fn fd_count_cur() -> Result<usize> {
    let count = platform::fd_count_cur()?;
    Ok(count as usize)
}
