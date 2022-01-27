use winapi::{
    shared::ntdef::FALSE,
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{GetCurrentProcess, GetProcessHandleCount, OpenProcess},
        winnt::{HANDLE, PROCESS_QUERY_INFORMATION},
    },
};

pub type Result<T> = std::io::Result<T>;

#[inline]
fn process_fd_count_inner(handler: ProcessHandler) -> Result<usize> {
    let mut count = 0;

    let ret = unsafe { GetProcessHandleCount(handler.raw(), &mut count) };
    if ret == 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(count as usize)
}

pub fn fd_count_pid(pid: u32) -> Result<usize> {
    let handler = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE as i32, pid) }.into();
    process_fd_count_inner(handler)
}

pub fn fd_count_cur() -> Result<usize> {
    let handler = unsafe { GetCurrentProcess() }.into();
    process_fd_count_inner(handler)
}

pub struct ProcessHandler(HANDLE);

impl ProcessHandler {
    pub fn raw(&self) -> HANDLE {
        self.0
    }
}

impl From<HANDLE> for ProcessHandler {
    fn from(h: HANDLE) -> Self {
        ProcessHandler(h)
    }
}

impl Drop for ProcessHandler {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use winapi::um::processthreadsapi::GetCurrentProcessId;

    #[test]
    fn test_count_fd() {
        const NUM: u32 = 100000;

        // open then close handle
        for _ in 0..NUM {
            let pid = unsafe { GetCurrentProcessId() };
            let handler = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE as i32, pid) };
            unsafe { CloseHandle(handler) };
        }
        let new_count = fd_count_cur().unwrap();

        assert!(new_count < NUM);

        // open some handle and do not close them
        for _ in 0..NUM {
            let pid = unsafe { GetCurrentProcessId() };
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE as i32, pid) };
        }
        let new_count = fd_count_cur().unwrap();

        assert!(new_count >= NUM);
    }
}
