//! Get cpu usage for current process and specified thread.
//!
//! A method named `cpu` on `ThreadStat` and `ProcessStat`
//! can retrieve cpu usage of thread and process respectively.
//!
//! The returning value is unnormalized, that is for multi-processor machine,
//! the cpu usage will beyond 100%, for example returning 2.8 means 280% cpu usage.
//! If normalized value is what you expected, divide the returning by processor_numbers.
//!
//! ## Example
//!
//! ```
//! # use perf_monitor::cpu::ThreadStat;
//! let mut stat = ThreadStat::cur().unwrap();
//! let _ = (0..1_000_000).into_iter().sum::<u64>();
//! let usage = stat.cpu().unwrap();
//! println!("current thread cpu usage is {:.2}%", usage * 100f64);
//! ```
//!
//! ## Bottom Layer Interface
//! | platform | thread | process |
//! | -- | -- | -- |
//! | windows |[GetThreadTimes] | [GetProcessTimes] |
//! | linux & android | [/proc/{pid}/task/{tid}/stat][man5] | [clockgettime] |
//! | macos & ios | [thread_info] | [getrusage] |
//!
//! [GetThreadTimes]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadtimes
//! [GetProcessTimes]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes
//! [man5]: https://man7.org/linux/man-pages/man5/proc.5.html
//! [thread_info]: http://web.mit.edu/darwin/src/modules/xnu/osfmk/man/thread_info.html
//! [clockgettime]: https://man7.org/linux/man-pages/man2/clock_gettime.2.html
//! [getrusage]: https://www.man7.org/linux/man-pages/man2/getrusage.2.html

#[cfg(any(target_os = "linux", target_os = "android"))]
mod android_linux;
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod ios_macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "linux", target_os = "android"))]
use android_linux as platform;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use ios_macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

pub use platform::{cpu_time, ThreadId};
pub use std::io::Result;
use std::{
    io, mem,
    time::{Duration, Instant},
};

/// logical processor number
pub fn processor_numbers() -> std::io::Result<usize> {
    std::thread::available_parallelism().map(|x| x.get())
}

/// A struct to monitor process cpu usage
pub struct ProcessStat {
    now: Instant,
    cpu_time: Duration,
}

impl ProcessStat {
    /// return a monitor of current process
    pub fn cur() -> io::Result<Self> {
        Ok(ProcessStat {
            now: Instant::now(),
            cpu_time: platform::cpu_time()?,
        })
    }

    /// return the cpu usage from last invoke,
    /// or when this struct created if it is the first invoke.
    pub fn cpu(&mut self) -> io::Result<f64> {
        let old_time = mem::replace(&mut self.cpu_time, platform::cpu_time()?);
        let old_now = mem::replace(&mut self.now, Instant::now());
        let real_time = self.now.saturating_duration_since(old_now).as_secs_f64();
        let cpu_time = self.cpu_time.saturating_sub(old_time).as_secs_f64();
        Ok(cpu_time / real_time)
    }
}

/// A struct to monitor thread cpu usage
pub struct ThreadStat {
    stat: platform::ThreadStat,
}

impl ThreadStat {
    /// return a monitor of current thread.
    pub fn cur() -> Result<Self> {
        Ok(ThreadStat {
            stat: platform::ThreadStat::cur()?,
        })
    }

    /// return a monitor of specified thread.
    ///
    /// `tid` is **NOT** `std::thread::ThreadId`.
    /// [`ThreadId::current`] can be used to retrieve a valid tid.
    pub fn build(thread_id: ThreadId) -> Result<Self> {
        Ok(ThreadStat {
            stat: platform::ThreadStat::build(thread_id)?,
        })
    }

    /// return the cpu usage from last invoke,
    /// or when this struct created if it is the first invoke.
    pub fn cpu(&mut self) -> Result<f64> {
        self.stat.cpu()
    }

    /// return the cpu_time in user mode and system mode from last invoke,
    /// or when this struct created if it is the first invoke.
    pub fn cpu_time(&mut self) -> Result<Duration> {
        self.stat.cpu_time()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // this test should be executed alone.
    #[test]
    #[ignore]
    fn test_process_usage() {
        let mut stat = ProcessStat::cur().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));

        let usage = stat.cpu().unwrap();

        assert!(usage < 0.01);

        let num = processor_numbers().unwrap();
        for _ in 0..num * 10 {
            std::thread::spawn(move || loop {
                let _ = (0..10_000_000).into_iter().sum::<u128>();
            });
        }

        let mut stat = ProcessStat::cur().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));

        let usage = stat.cpu().unwrap();

        assert!(usage > 0.9 * num as f64)
    }

    #[test]
    fn test_thread_usage() {
        let mut stat = ThreadStat::cur().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
        let usage = stat.cpu().unwrap();
        assert!(usage < 0.01);

        let mut x = 1_000_000u64;
        std::hint::black_box(&mut x);
        let mut times = 1000u64;
        std::hint::black_box(&mut times);
        for i in 0..times {
            let x = (0..x + i).into_iter().sum::<u64>();
            std::hint::black_box(x);
        }
        let usage = stat.cpu().unwrap();
        assert!(usage > 0.5)
    }
}
