use std::io::Error;
use std::io::Result;
use std::mem::MaybeUninit;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetProcessTimes;

pub struct ProcessTimes {
    pub create: FILETIME,
    pub exit: FILETIME,
    pub kernel: FILETIME,
    pub user: FILETIME,
}

impl ProcessTimes {
    pub fn capture_current() -> Result<Self> {
        unsafe { Self::capture_with_handle(GetCurrentProcess()) }
    }

    pub unsafe fn capture_with_handle(handle: HANDLE) -> Result<Self> {
        let mut create = MaybeUninit::<FILETIME>::uninit();
        let mut exit = MaybeUninit::<FILETIME>::uninit();
        let mut kernel = MaybeUninit::<FILETIME>::uninit();
        let mut user = MaybeUninit::<FILETIME>::uninit();
        let ret = unsafe {
            GetProcessTimes(
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
