use super::ThreadId;
use crate::utils::ptr_upgrade::HandleUpgrade;
use crate::utils::windows_handle::Handle;
use std::io::Error;
use std::io::Result;
use std::mem::MaybeUninit;
use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::GetCurrentThread;
use windows_sys::Win32::System::Threading::GetThreadTimes;
use windows_sys::Win32::System::Threading::OpenThread;
use windows_sys::Win32::System::Threading::THREAD_QUERY_LIMITED_INFORMATION;

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
        unsafe { Self::capture_with_handle(handle.as_handle()) }
    }

    /// Get thread times for given thread handle, given handle needs specific access rights:
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadtimes
    pub unsafe fn capture_with_handle(handle: HANDLE) -> Result<Self> {
        let mut create = MaybeUninit::<FILETIME>::uninit();
        let mut exit = MaybeUninit::<FILETIME>::uninit();
        let mut kernel = MaybeUninit::<FILETIME>::uninit();
        let mut user = MaybeUninit::<FILETIME>::uninit();
        let ret = unsafe {
            GetThreadTimes(
                handle,
                create.as_mut_ptr(),
                exit.as_mut_ptr(),
                kernel.as_mut_ptr(),
                user.as_mut_ptr(),
            )
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }
        Ok(unsafe {
            Self {
                create: create.assume_init(),
                exit: exit.assume_init(),
                kernel: kernel.assume_init(),
                user: user.assume_init(),
            }
        })
    }
}
