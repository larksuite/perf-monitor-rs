#[cfg(any(target_os = "ios", target_os = "macos"))]
mod tests {
    use criterion::Criterion;
    use libc::time_value_t;
    use perf_monitor::{
        cpu::ThreadId,
        cpu::get_thread_basic_info
    };
    use std::{
        convert::TryInto,
        time::Instant,
        time::Duration
    };

    #[inline]
    fn time_value_to_u64(t: time_value_t) -> u64 {
        (t.seconds.try_into().unwrap_or(0u64))
            .saturating_mul(1_000_000)
            .saturating_add(t.microseconds.try_into().unwrap_or(0u64))
    }
    // There is a field named `cpu_usage` in `thread_basic_info` which represents the CPU usage of the thread.
    // However, we have no idea about how long the interval is. And it will make the API being different from other platforms.
    // We calculate the usage instead of using the field directory to make the API is the same on all platforms.
    // The cost of the calculation is very very small according to the result of the following benchmark.
    pub fn bench_cpu_usage_by_calculate(c: &mut Criterion) {
        c.bench_function("CPU usage by calculate", |b| {
            let tid = ThreadId::current();
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
        });
    }

    pub fn bench_cpu_usage_by_field(c: &mut Criterion) {
        c.bench_function("CPU usage by field", |b| {
            let tid = ThreadId::current();
            b.iter(|| {
                let cur_stat = get_thread_basic_info(tid).unwrap();
                let _ = cur_stat.cpu_usage / 1000;
            });
        });
    }
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
criterion::criterion_group!(
    benches,
    tests::bench_cpu_usage_by_calculate,
    tests::bench_cpu_usage_by_field
);
#[cfg(any(target_os = "ios", target_os = "macos"))]
criterion::criterion_main!(benches);

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn main() {
    println!("This benchmark can only be run on iOS or MacOS.");
}