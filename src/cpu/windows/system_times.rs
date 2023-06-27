use std::io::Error;
use std::io::Result;
use std::mem::MaybeUninit;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::GetSystemTimes;

pub struct SystemTimes {
    pub idle: FILETIME,
    pub kernel: FILETIME,
    pub user: FILETIME,
}

impl SystemTimes {
    pub fn capture() -> Result<Self> {
        let mut idle = MaybeUninit::<FILETIME>::uninit();
        let mut kernel = MaybeUninit::<FILETIME>::uninit();
        let mut user = MaybeUninit::<FILETIME>::uninit();
        let ret =
            unsafe { GetSystemTimes(idle.as_mut_ptr(), kernel.as_mut_ptr(), user.as_mut_ptr()) };
        if ret == 0 {
            return Err(Error::last_os_error());
        }
        Ok(unsafe {
            Self {
                idle: idle.assume_init(),
                kernel: kernel.assume_init(),
                user: user.assume_init(),
            }
        })
    }
}
