use super::ThreadId;
use crate::utils::ptr_upgrade::PointerUpgrade;
use crate::utils::windows_handle::Handle;
use std::io::Error;
use std::io::Result;
use std::os::raw::c_void;
use winapi::um::processthreadsapi::GetCurrentThread;
use winapi::{
    shared::{minwindef::FILETIME, ntdef::FALSE},
    um::{
        processthreadsapi::{GetThreadTimes, OpenThread},
        winnt::THREAD_QUERY_LIMITED_INFORMATION,
    },
};

#[derive(Default)]
pub struct ThreadTimes {
    pub create: FILETIME,
    pub exit: FILETIME,
    pub kernel: FILETIME,
    pub user: FILETIME,
}

impl ThreadTimes {
    #[allow(dead_code)]
    pub fn capture_current() -> Result<Self> {
        unsafe { Self::capture_with_handle(GetCurrentThread()) }
    }

    pub fn capture_with_thread_id(ThreadId(thread_id): ThreadId) -> Result<Self> {
        // Use THREAD_QUERY_LIMITED_INFORMATION to acquire minimum access rights and
        // support for Windows Server 2023 and Windows XP is dropped:
        //
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadtimes
        let handle =
            unsafe { OpenThread(THREAD_QUERY_LIMITED_INFORMATION, FALSE as i32, thread_id) }
                .upgrade()
                .map(|x| unsafe { Handle::new(x) });
        let Some(handle) = handle else {
            return Err(Error::last_os_error());
        };
        unsafe { Self::capture_with_handle(handle.as_ptr()) }
    }

    /// Get thread times for given thread handle, given handle needs specific access rights:
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadtimes
    pub unsafe fn capture_with_handle(handle: *mut c_void) -> Result<Self> {
        let mut thread_times = ThreadTimes::default();
        let ret = unsafe {
            GetThreadTimes(
                handle,
                &mut thread_times.create,
                &mut thread_times.exit,
                &mut thread_times.kernel,
                &mut thread_times.user,
            )
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }
        Ok(thread_times)
    }
}
