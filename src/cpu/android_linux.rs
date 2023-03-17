use libc::{syscall, sysconf, SYS_gettid, _SC_CLK_TCK};
use lpfs::{pid::stat_of_task, proc::cpuinfo, ProcErr};
use std::{
    io,
    time::{Duration, Instant},
};

/// logical processor number
pub fn processor_numbers() -> io::Result<usize> {
    let cpu_info = cpuinfo().map_err(conv_err)?;
    Ok(cpu_info.physical_core_num())
}

#[inline]
fn cur_process_id() -> u32 {
    std::process::id()
}

#[inline]
pub fn cur_thread_id() -> io::Result<u32> {
    let ret = unsafe { syscall(SYS_gettid) };
    Ok(ret as u32)
}

pub fn clock_tick() -> i64 {
    unsafe { sysconf(_SC_CLK_TCK) as i64 }
}

pub struct ThreadStat {
    pid: u32,
    tid: u32,
    last_stat: (u64, Instant),
}

impl ThreadStat {
    pub fn cur() -> io::Result<Self> {
        let tid = cur_thread_id()?;
        let pid = cur_process_id();
        Self::build(pid, tid)
    }

    pub fn build(pid: u32, tid: u32) -> io::Result<Self> {
        let s = stat_of_task(pid, tid).map_err(conv_err)?;
        let wtime = *s.utime() + *s.stime() + *s.cutime() as u64 + *s.cstime() as u64;
        let total_time = Instant::now();

        Ok(ThreadStat {
            pid,
            tid,
            last_stat: (wtime, total_time),
        })
    }

    /// un-normalized
    pub fn cpu(&mut self) -> io::Result<f64> {
        let s = stat_of_task(self.pid, self.tid).map_err(conv_err)?;
        let wtime = *s.utime() + *s.stime() + *s.cutime() as u64 + *s.cstime() as u64;
        let total_time = Instant::now();

        let dt_total = (total_time - self.last_stat.1).as_millis();
        if dt_total == 0 {
            return Ok(0.0);
        }
        let dt_worktime = (wtime - self.last_stat.0) * 1000 / clock_tick() as u64;

        self.last_stat = (wtime, total_time);

        Ok(dt_worktime as f64 / dt_total as f64)
    }

    pub fn cpu_time(&mut self) -> io::Result<Duration> {
        let s = stat_of_task(self.pid, self.tid).map_err(conv_err)?;
        let wtime = *s.utime() + *s.stime() + *s.cutime() as u64 + *s.cstime() as u64;
        let total_time = Instant::now();

        let dt_worktime = (wtime - self.last_stat.0) * 1000 / clock_tick() as u64;

        self.last_stat = (wtime, total_time);

        Ok(Duration::from_millis(dt_worktime))
    }
}

pub fn cpu_time() -> io::Result<Duration> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    if unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut time) } == 0 {
        Ok(Duration::new(time.tv_sec as u64, time.tv_nsec as u32))
    } else {
        Err(io::Error::last_os_error())
    }
}

fn conv_err(err: ProcErr) -> io::Error {
    match err {
        ProcErr::IO(err) => err,
        ProcErr::Parse(err) => io::Error::new(io::ErrorKind::InvalidData, format!("{}", err)),
        ProcErr::BadFormat(err) => io::Error::new(io::ErrorKind::InvalidData, err),
    }
}

#[cfg(test)]
#[allow(clippy::all)]
mod test {
    use super::*;

    #[test]
    fn test_clock_tick() {
        assert_eq!(100, clock_tick())
    }
}
