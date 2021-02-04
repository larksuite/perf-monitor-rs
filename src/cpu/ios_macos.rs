use crate::bindings::*;
use std::time::Instant;
use std::{io, mem::size_of, time::Duration};

fn self_host() -> u32 {
    unsafe { mach_host_self() }
}

/// logic processor number
pub fn processor_numbers() -> io::Result<i32> {
    let host = self_host();
    let flavor = HOST_BASIC_INFO as i32;
    let mut host_info_ = host_basic_info::default();
    let mut host_info_cnt = size_of::<host_basic_info>() as u32;

    let ret = unsafe {
        host_info(
            host,
            flavor,
            (&mut host_info_ as *mut host_basic_info) as *mut i32,
            &mut host_info_cnt,
        )
    };
    if ret != KERN_SUCCESS as i32 {
        return Err(io::Error::from_raw_os_error(ret));
    }

    Ok(host_info_.logical_cpu)
}

fn get_thread_basic_info(tid: u32) -> io::Result<thread_basic_info> {
    let flavor = THREAD_BASIC_INFO;
    let mut thread_basic_info_ = thread_basic_info::default();
    let mut thread_info_cnt = size_of::<thread_basic_info>() as u32;

    let ret = unsafe {
        thread_info(
            tid,
            flavor,
            (&mut thread_basic_info_ as *mut thread_basic_info) as *mut i32,
            &mut thread_info_cnt,
        )
    };
    if ret != KERN_SUCCESS as i32 {
        return Err(io::Error::from_raw_os_error(ret));
    }

    Ok(thread_basic_info_)
}

pub struct ThreadStat {
    tid: u32,
    stat: (thread_basic_info, Instant),
}

impl ThreadStat {
    pub fn cur() -> io::Result<Self> {
        Self::build(cur_thread_id()?)
    }

    pub fn build(tid: u32) -> io::Result<Self> {
        Ok(ThreadStat {
            tid,
            stat: (get_thread_basic_info(tid)?, Instant::now()),
        })
    }

    /// unnormalized
    pub fn cpu(&mut self) -> io::Result<f64> {
        let (last_stat, last_time) = self.stat;
        let cur_stat = get_thread_basic_info(self.tid)?;
        let cur_time = Instant::now();

        let cur_user_time = time_value_to_u64(cur_stat.user_time);
        let cur_sys_time = time_value_to_u64(cur_stat.system_time);
        let last_user_time = time_value_to_u64(last_stat.user_time);
        let last_sys_time = time_value_to_u64(last_stat.system_time);

        let dt_duration = cur_time - last_time;
        let cpu_time_us = cur_user_time + cur_sys_time - last_user_time - last_sys_time;
        let dt_wtime = Duration::from_micros(cpu_time_us);

        self.stat = (cur_stat, cur_time);
        Ok(dt_wtime.as_micros() as f64 / dt_duration.as_micros() as f64)
    }

    pub fn cpu_time(&mut self) -> io::Result<Duration> {
        let cur_stat = get_thread_basic_info(self.tid)?;

        let cur_user_time = time_value_to_u64(cur_stat.user_time);
        let cur_sys_time = time_value_to_u64(cur_stat.system_time);
        let last_user_time = time_value_to_u64(self.stat.0.user_time);
        let last_sys_time = time_value_to_u64(self.stat.0.system_time);

        let cpu_time_us = cur_user_time + cur_sys_time - last_user_time - last_sys_time;
        let cpu_time = Duration::from_micros(cpu_time_us);

        self.stat = (cur_stat, Instant::now());
        Ok(cpu_time)
    }
}

#[inline]
fn time_value_to_u64(t: time_value) -> u64 {
    (t.seconds as u64) * 1_000_000u64 + (t.microseconds as u64)
}

#[inline]
pub fn cur_thread_id() -> io::Result<u32> {
    Ok(unsafe { mach_thread_self() })
}

// The `clock_gettime` is not supported in older version of mac/ios before 2016, so `getrusage` is used instead.
//
// `times` is not used, becasuse it's returning value is clock ticks instead of time, lowwer accuracy, different with other platform and deprecated.
//
// `getrusage` is about 100ns slowwer than `clock_Gettime` each round.
pub fn cpu_time() -> io::Result<Duration> {
    let mut time = unsafe { std::mem::zeroed() };

    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut time) } == 0 {
        let sec = time.ru_utime.tv_sec as u64 + time.ru_stime.tv_sec as u64;
        let nsec = (time.ru_utime.tv_usec as u32 + time.ru_stime.tv_usec as u32) * 1000;

        Ok(Duration::new(sec, nsec))
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
#[allow(clippy::all, clippy::print_stdout)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_processor_num() {
        let t = processor_numbers().unwrap();
        println!("processor num: {:?}", t);
        assert!(t >= 1);
    }

    // There is a field named `cpu_usage` in `thread_basic_info` which represents the CPU usage of the thread.
    // However, we have no idea about how long the interval is. And it will make the API being different from other platforms.
    // We calculate the usage instead of using the field directory to make the API is the same on all platforms.
    // The cost of the calculation is very very small according to the result of the following benchmark.
    #[bench]
    fn bench_cpu_usage_by_calculate(b: &mut Bencher) {
        let tid = cur_thread_id().unwrap();
        let last_stat = get_thread_basic_info(tid).unwrap();
        let last_time = Instant::now();

        b.iter(|| {
            let cur_stat = get_thread_basic_info(tid).unwrap();
            let cur_time = Instant::now();

            let cur_user_time = time_value_to_u64(cur_stat.user_time);
            let cur_sys_time = time_value_to_u64(cur_stat.system_time);
            let last_user_time = time_value_to_u64(last_stat.user_time);
            let last_sys_time = time_value_to_u64(last_stat.system_time);

            let dt_duration = cur_time - last_time;
            let cpu_time_us = cur_user_time + cur_sys_time - last_user_time - last_sys_time;
            let dt_wtime = Duration::from_micros(cpu_time_us);

            let _ = (cur_stat, cur_time);
            let _ = dt_wtime.as_micros() as f64 / dt_duration.as_micros() as f64;
        });
    }

    #[bench]
    fn bench_cpu_usage_by_field(b: &mut Bencher) {
        let tid = cur_thread_id().unwrap();
        b.iter(|| {
            let cur_stat = get_thread_basic_info(tid).unwrap();
            let _ = cur_stat.cpu_usage / 1000;
        });
    }
}
