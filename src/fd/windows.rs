use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetProcessHandleCount;
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

use crate::utils::ptr_upgrade::HandleUpgrade;
use crate::utils::windows_handle::Handle;

#[inline]
fn process_fd_count(handler: HANDLE) -> std::io::Result<u32> {
    let mut count = 0;
    let ret = unsafe { GetProcessHandleCount(handler, &mut count) };
    if ret == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(count)
}

pub fn fd_count_pid(pid: u32) -> std::io::Result<u32> {
    // Use PROCESS_QUERY_LIMITED_INFORMATION to acquire less privilege and drop
    // support for Windows Server 2023 and Windows XP:
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesshandlecount
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE as i32, pid) }
        .upgrade()
        .map(|x| unsafe { Handle::new(x) });
    let Some(handle) = handle else {
        return Err(std::io::Error::last_os_error());
    };
    process_fd_count(handle.as_handle())
}

pub fn fd_count_cur() -> std::io::Result<u32> {
    process_fd_count(unsafe { GetCurrentProcess() })
}

#[cfg(test)]
mod test {
    use super::*;
    use windows_sys::Win32::Foundation::CloseHandle;

    #[test]
    fn test_count_fd() {
        const NUM: u32 = 100000;

        // open then close handle
        for _ in 0..NUM {
            let pid = std::process::id();
            let handler =
                unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE as i32, pid) };
            unsafe { CloseHandle(handler) };
        }
        let new_count = fd_count_cur().unwrap();

        assert!(new_count < NUM);

        // open some handle and do not close them
        for _ in 0..NUM {
            let pid = std::process::id();
            unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE as i32, pid) };
        }
        let new_count = fd_count_cur().unwrap();

        assert!(new_count >= NUM);
    }
}
