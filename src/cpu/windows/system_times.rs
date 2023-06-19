use std::io::Error;
use std::io::Result;
use winapi::{shared::minwindef::FILETIME, um::processthreadsapi::GetSystemTimes};

#[derive(Default)]
pub struct SystemTimes {
    pub idle: FILETIME,
    pub kernel: FILETIME,
    pub user: FILETIME,
}

impl SystemTimes {
    pub fn capture() -> Result<Self> {
        let mut times = Self::default();
        let ret = unsafe { GetSystemTimes(&mut times.idle, &mut times.kernel, &mut times.user) };
        if ret == 0 {
            return Err(Error::last_os_error());
        }
        Ok(times)
    }
}
