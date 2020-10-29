use scopeguard::defer;
use std::{io, mem, time::Duration};
use winapi::{
    shared::{
        minwindef::FILETIME,
        ntdef::{FALSE, NULL},
    },
    um::{
        handleapi::CloseHandle,
        processthreadsapi::{
            GetCurrentProcess, GetCurrentThreadId, GetProcessTimes, GetSystemTimes, GetThreadTimes,
            OpenThread,
        },
        sysinfoapi::{GetSystemInfo, SYSTEM_INFO},
        winnt::THREAD_QUERY_INFORMATION,
    },
};

/// convert to u64, unit 100 ns
fn filetime_to_ns100(ft: FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) + ft.dwLowDateTime as u64
}

fn get_sys_times() -> io::Result<(u64, u64, u64)> {
    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();

    let ret = unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
    if ret == 0 {
        return Err(io::Error::last_os_error());
    }

    let idle = filetime_to_ns100(idle);
    let kernel = filetime_to_ns100(kernel);
    let user = filetime_to_ns100(user);
    Ok((idle, kernel, user))
}

fn get_thread_times(tid: u32) -> io::Result<(u64, u64)> {
    let handler = unsafe { OpenThread(THREAD_QUERY_INFORMATION, FALSE as i32, tid) };
    if handler == NULL {
        return Err(io::Error::last_os_error());
    }
    defer! {{
        unsafe { CloseHandle(handler) };
    }}

    let mut create_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();

    let ret = unsafe {
        GetThreadTimes(
            handler,
            &mut create_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )
    };
    if ret == 0 {
        return Err(io::Error::last_os_error());
    }

    // let create_time = filetime_to_ns100(create_time);
    // let exit_time = filetime_to_ns100(exit_time);
    let kernel_time = filetime_to_ns100(kernel_time);
    let user_time = filetime_to_ns100(user_time);
    Ok((kernel_time, user_time))
}

#[inline]
pub fn cur_thread_id() -> io::Result<u32> {
    let handler = unsafe { GetCurrentThreadId() };
    Ok(handler)
}

#[inline]
pub fn processor_numbers() -> io::Result<usize> {
    let mut sysinfo = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut sysinfo) };
    Ok(sysinfo.dwNumberOfProcessors as usize)
}

pub struct ThreadStat {
    tid: u32,
    last_work_time: u64,
    last_total_time: u64,
}

impl ThreadStat {
    fn get_times(tid: u32) -> io::Result<(u64, u64)> {
        let sys_time = get_sys_times()?;
        let pro_time = get_thread_times(tid)?;

        let pwork_time = pro_time.0 + pro_time.1;
        let total_time = sys_time.1 + sys_time.2;
        Ok((pwork_time, total_time))
    }

    pub fn cur() -> io::Result<Self> {
        let tid = cur_thread_id()?;
        let (work_time, total_time) = Self::get_times(tid)?;
        Ok(ThreadStat {
            tid,
            last_work_time: work_time,
            last_total_time: total_time,
        })
    }

    pub fn build(tid: u32) -> io::Result<Self> {
        let (work_time, total_time) = Self::get_times(tid)?;
        Ok(ThreadStat {
            tid,
            last_work_time: work_time,
            last_total_time: total_time,
        })
    }

    pub fn cpu(&mut self) -> io::Result<f64> {
        let (work_time, total_time) = Self::get_times(self.tid)?;

        let dt_total_time = total_time - self.last_total_time;
        if dt_total_time == 0 {
            return Ok(0.0);
        }
        let dt_work_time = work_time - self.last_work_time;

        self.last_work_time = work_time;
        self.last_total_time = total_time;

        Ok(dt_work_time as f64 / dt_total_time as f64 * processor_numbers()? as f64)
    }

    pub fn cpu_time(&mut self) -> io::Result<Duration> {
        let (work_time, total_time) = Self::get_times(self.tid)?;

        let cpu_time = work_time - self.last_work_time;

        self.last_work_time = work_time;
        self.last_total_time = total_time;

        Ok(Duration::from_nanos(cpu_time))
    }
}

#[inline]
pub fn cpu_time() -> io::Result<Duration> {
    let (kernel_time, user_time) = unsafe {
        let process = GetCurrentProcess();
        let mut create_time = mem::zeroed();
        let mut exit_time = mem::zeroed();
        let mut kernel_time = mem::zeroed();
        let mut user_time = mem::zeroed();

        let ret = GetProcessTimes(
            process,
            &mut create_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        );

        if ret != 0 {
            (kernel_time, user_time)
        } else {
            return Err(io::Error::last_os_error());
        }
    };

    let kt = filetime_to_ns100(kernel_time);
    let ut = filetime_to_ns100(user_time);

    // convert ns
    //
    // Note: make it ns unit may overflow in some cases.
    // For example, a machine with 128 cores runs for one year.
    let cpu = (kt + ut) * 100;

    // make it un-normalized
    let cpu = cpu * processor_numbers()? as u64;

    Ok(Duration::from_nanos(cpu))
}

#[cfg(test)]
#[allow(clippy::all)]
mod test {
    use super::*;

    #[test]
    fn test_processor_number() {
        let n = processor_numbers().unwrap();
        assert!(n >= 1)
    }
}
