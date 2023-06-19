use super::processor_numbers;
use super::windows::process_times::ProcessTimes;
use super::windows::system_times::SystemTimes;
use super::windows::thread_times::ThreadTimes;
use std::io::Result;
use std::time::Duration;
use winapi::{shared::minwindef::FILETIME, um::processthreadsapi::GetCurrentThreadId};

pub mod process_times;
pub mod system_times;
pub mod thread_times;

#[derive(Clone, Copy)]
pub struct ThreadId(u32);

impl ThreadId {
    #[inline]
    pub fn current() -> Self {
        ThreadId(unsafe { GetCurrentThreadId() })
    }
}

/// convert to u64, unit 100 ns
fn filetime_to_ns100(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) + ft.dwLowDateTime as u64
}

pub struct ThreadStat {
    tid: ThreadId,
    last_work_time: u64,
    last_total_time: u64,
}

impl ThreadStat {
    fn get_times(thread_id: ThreadId) -> Result<(u64, u64)> {
        let system_times = SystemTimes::capture()?;
        let thread_times = ThreadTimes::capture_with_thread_id(thread_id)?;

        let work_time =
            filetime_to_ns100(&thread_times.kernel) + filetime_to_ns100(&thread_times.user);
        let total_time =
            filetime_to_ns100(&system_times.kernel) + filetime_to_ns100(&system_times.user);
        Ok((work_time, total_time))
    }

    pub fn cur() -> Result<Self> {
        let tid = ThreadId::current();
        let (work_time, total_time) = Self::get_times(tid)?;
        Ok(ThreadStat {
            tid,
            last_work_time: work_time,
            last_total_time: total_time,
        })
    }

    pub fn build(tid: ThreadId) -> Result<Self> {
        let (work_time, total_time) = Self::get_times(tid)?;
        Ok(ThreadStat {
            tid,
            last_work_time: work_time,
            last_total_time: total_time,
        })
    }

    pub fn cpu(&mut self) -> Result<f64> {
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

    pub fn cpu_time(&mut self) -> Result<Duration> {
        let (work_time, total_time) = Self::get_times(self.tid)?;

        let cpu_time = work_time - self.last_work_time;

        self.last_work_time = work_time;
        self.last_total_time = total_time;

        Ok(Duration::from_nanos(cpu_time))
    }
}

#[inline]
pub fn cpu_time() -> Result<Duration> {
    let process_times = ProcessTimes::capture_current()?;

    let kt = filetime_to_ns100(&process_times.kernel);
    let ut = filetime_to_ns100(&process_times.user);

    // convert ns
    //
    // Note: make it ns unit may overflow in some cases.
    // For example, a machine with 128 cores runs for one year.
    let cpu = (kt + ut).saturating_mul(100);

    // make it un-normalized
    let cpu = cpu * processor_numbers()? as u64;

    Ok(Duration::from_nanos(cpu))
}
