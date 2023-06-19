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

#[cfg(all(target_os = "macos", not(feature = "darwin_private")))]
mod macos;
#[cfg(all(target_os = "macos", not(feature = "darwin_private")))]
use macos as platform;

#[cfg(all(target_os = "ios", not(feature = "darwin_private")))]
mod ios;
#[cfg(all(target_os = "ios", not(feature = "darwin_private")))]
use ios as platform;

#[cfg(all(
    any(target_os = "macos", target_os = "ios"),
    feature = "darwin_private"
))]
mod darwin_private;
#[cfg(all(
    any(target_os = "macos", target_os = "ios"),
    feature = "darwin_private"
))]
use darwin_private as platform;

/// return the fd count of current process
#[inline]
pub fn fd_count_cur() -> std::io::Result<usize> {
    platform::fd_count_cur().map(|count| count as usize)
}
