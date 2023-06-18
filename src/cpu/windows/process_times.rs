use libc::c_void;
use std::io::Error;
use std::io::Result;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::{shared::minwindef::FILETIME, um::processthreadsapi::GetProcessTimes};

#[derive(Default)]
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

    pub unsafe fn capture_with_handle(handle: *mut c_void) -> Result<Self> {
        let mut process_times = ProcessTimes::default();
        let ret = unsafe {
            GetProcessTimes(
                handle,
                &mut process_times.create,
                &mut process_times.exit,
                &mut process_times.kernel,
                &mut process_times.user,
            )
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }
        Ok(process_times)
    }
}
