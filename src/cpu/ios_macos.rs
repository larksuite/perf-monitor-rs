use libc::{
    mach_thread_self, rusage, thread_basic_info, time_value_t, KERN_SUCCESS, RUSAGE_SELF,
    THREAD_BASIC_INFO, THREAD_BASIC_INFO_COUNT,
};
use std::convert::TryInto;
use std::mem::MaybeUninit;
use std::time::Instant;
use std::{
    io::{Error, Result},
    time::Duration,
};

#[derive(Clone, Copy)]
pub struct ThreadId(u32);

impl ThreadId {
    #[inline]
    pub fn current() -> Self {
        ThreadId(unsafe { mach_thread_self() })
    }
}

fn get_thread_basic_info(ThreadId(tid): ThreadId) -> Result<thread_basic_info> {
    let mut thread_basic_info = MaybeUninit::<thread_basic_info>::uninit();
    let mut thread_info_cnt = THREAD_BASIC_INFO_COUNT;

    let ret = unsafe {
        libc::thread_info(
            tid,
            THREAD_BASIC_INFO as u32,
            thread_basic_info.as_mut_ptr() as *mut _,
            &mut thread_info_cnt,
        )
    };
    if ret != KERN_SUCCESS as i32 {
        return Err(Error::from_raw_os_error(ret));
    }
    Ok(unsafe { thread_basic_info.assume_init() })
}

pub struct ThreadStat {
    tid: ThreadId,
    stat: (thread_basic_info, Instant),
}

impl ThreadStat {
    pub fn cur() -> Result<Self> {
        Self::build(ThreadId::current())
    }

    pub fn build(tid: ThreadId) -> Result<Self> {
        Ok(ThreadStat {
            tid,
            stat: (get_thread_basic_info(tid)?, Instant::now()),
        })
    }

    /// unnormalized
    pub fn cpu(&mut self) -> Result<f64> {
        let cur_stat = get_thread_basic_info(self.tid)?;
        let cur_time = Instant::now();
        let (last_stat, last_time) = std::mem::replace(&mut self.stat, (cur_stat, cur_time));

        let cur_user_time = time_value_to_u64(cur_stat.user_time);
        let cur_sys_time = time_value_to_u64(cur_stat.system_time);
        let last_user_time = time_value_to_u64(last_stat.user_time);
        let last_sys_time = time_value_to_u64(last_stat.system_time);

        let cpu_time_us = cur_user_time
            .saturating_sub(last_user_time)
            .saturating_add(cur_sys_time.saturating_sub(last_sys_time));

        let dt_duration = cur_time.saturating_duration_since(last_time);
        Ok(cpu_time_us as f64 / dt_duration.as_micros() as f64)
    }

    pub fn cpu_time(&mut self) -> Result<Duration> {
        let cur_stat = get_thread_basic_info(self.tid)?;
        let cur_time = Instant::now();
        let (last_stat, _last_time) = std::mem::replace(&mut self.stat, (cur_stat, cur_time));

        let cur_user_time = time_value_to_u64(cur_stat.user_time);
        let cur_sys_time = time_value_to_u64(cur_stat.system_time);
        let last_user_time = time_value_to_u64(last_stat.user_time);
        let last_sys_time = time_value_to_u64(last_stat.system_time);

        let cpu_time_us = cur_user_time
            .saturating_sub(last_user_time)
            .saturating_add(cur_sys_time.saturating_sub(last_sys_time));

        Ok(Duration::from_micros(cpu_time_us))
    }
}

#[inline]
fn time_value_to_u64(t: time_value_t) -> u64 {
    (t.seconds.try_into().unwrap_or(0u64))
        .saturating_mul(1_000_000)
        .saturating_add(t.microseconds.try_into().unwrap_or(0u64))
}

pub fn cpu_time() -> Result<Duration> {
    let mut time = MaybeUninit::<rusage>::uninit();
    let ret = unsafe { libc::getrusage(RUSAGE_SELF, time.as_mut_ptr()) };
    if ret != 0 {
        return Err(Error::last_os_error());
    }
    let time = unsafe { time.assume_init() };
    let sec = (time.ru_utime.tv_sec as u64).saturating_add(time.ru_stime.tv_sec as u64);
    let nsec = (time.ru_utime.tv_usec as u32)
        .saturating_add(time.ru_stime.tv_usec as u32)
        .saturating_mul(1000);
    Ok(Duration::new(sec, nsec))
}
