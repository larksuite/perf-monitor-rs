use libc::{timespec, pthread_t};
use std::{
    convert::TryInto,
    io::Error,
    io::Result,
    mem::MaybeUninit,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub struct ThreadId(pthread_t);

impl ThreadId {
    #[inline]
    pub fn current() -> Self {
        ThreadId(unsafe { libc::pthread_self() })
    }
}

fn timespec_to_duration(timespec { tv_sec, tv_nsec }: timespec) -> Duration {
    let sec: u64 = tv_sec.try_into().unwrap_or_default();
    let nsec: u64 = tv_nsec.try_into().unwrap_or_default();
    let (sec, nanos) = (
        sec.saturating_add(nsec / 1_000_000_000),
        (nsec % 1_000_000_000) as u32,
    );
    Duration::new(sec, nanos)
}

fn get_thread_cputime(ThreadId(thread): ThreadId) -> Result<timespec> {
    let mut clk_id = 0;
    let ret = unsafe { libc::pthread_getcpuclockid(thread, &mut clk_id) };
    if ret != 0 {
        return Err(Error::from_raw_os_error(ret));
    }

    let mut timespec = MaybeUninit::<timespec>::uninit();
    let ret = unsafe { libc::clock_gettime(clk_id, timespec.as_mut_ptr()) };
    if ret != 0 {
        return Err(Error::last_os_error());
    }
    Ok(unsafe { timespec.assume_init() })
}

pub struct ThreadStat {
    tid: ThreadId,
    last_stat: (timespec, Instant),
}

impl ThreadStat {
    pub fn cur() -> Result<Self> {
        Self::build(ThreadId::current())
    }

    pub fn build(tid: ThreadId) -> Result<Self> {
        let cputime = get_thread_cputime(tid)?;
        let total_time = Instant::now();
        Ok(ThreadStat {
            tid,
            last_stat: (cputime, total_time),
        })
    }

    /// un-normalized
    pub fn cpu(&mut self) -> Result<f64> {
        let cputime = get_thread_cputime(self.tid)?;
        let total_time = Instant::now();
        let (old_cputime, old_total_time) =
            std::mem::replace(&mut self.last_stat, (cputime, total_time));
        let cputime = cputime.tv_sec as f64 + cputime.tv_nsec as f64 / 1_000_000_000f64;
        let old_cputime = old_cputime.tv_sec as f64 + old_cputime.tv_nsec as f64 / 1_000_000_000f64;
        let dt_cputime = cputime - old_cputime;
        let dt_total_time = total_time
            .saturating_duration_since(old_total_time)
            .as_secs_f64();
        Ok(dt_cputime / dt_total_time)
    }

    pub fn cpu_time(&mut self) -> Result<Duration> {
        let cputime = get_thread_cputime(self.tid)?;
        let total_time = Instant::now();
        let (old_cputime, _old_total_time) =
            std::mem::replace(&mut self.last_stat, (cputime, total_time));
        Ok(timespec_to_duration(timespec {
            tv_sec: cputime.tv_sec - old_cputime.tv_sec,
            tv_nsec: cputime.tv_nsec - old_cputime.tv_nsec,
        }))
    }
}

pub fn cpu_time() -> Result<Duration> {
    let mut timespec = MaybeUninit::<timespec>::uninit();
    let ret = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, timespec.as_mut_ptr()) };
    if ret != 0 {
        return Err(Error::last_os_error());
    }
    Ok(timespec_to_duration(unsafe { timespec.assume_init() }))
}
