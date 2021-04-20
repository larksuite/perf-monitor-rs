# perf-monitor-rs

[![github](https://img.shields.io/badge/GitHub-perf_monitor_rs-9b88bb?logo=github)](https://github.com/larksuite/perf-monitor-rs)
[![minimum rustc 1.31.0](https://img.shields.io/badge/Minimum%20rustc-1.31.0-c18170?logo=rust)](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![docs.rs](https://docs.rs/perf_monitor/badge.svg)](https://docs.rs/perf_monitor)
[![crates.io](https://img.shields.io/crates/v/perf_monitor.svg)](https://crates.io/crates/perf_monitor)

```toml
# Cargo.toml
[dependencies]
perf_monitor = "0.2"
```

A toolkit designed to be a foundation for applications to monitor their performance. It is:
- **Cross-Platform:** perf-monitor supports Windows, macOS, Linux, iOS, and Android.
- **Safe Wrapper:** perf-monitor uses many system C interfaces internally but exposes safe wrapper API outside. 
- **Effective:** perf-monitor is a thin wrapper around underlying APIs, taking care of not introducing unnecessary overhead, choosing the most lightweight method among many homogeneous APIs.

# Features
- CPU
    - Usage of current process
    - Usage of other process (coming soon)
    - Usage of any thread in current process
    - Logic core number
- Memory
    - A lobal allocator that tracks rust allocations
    - Process memory info of current process for Windows and MacOS(Linux is conming soon).
- IO
    - Disk IO
    - Network IO(coming soon)
- FD
    - FD number

# Example
A simple activity monitor:

```rust
    use perf_monitor::cpu::{ThreadStat, ProcessStat, processor_numbers};
    use perf_monitor::fd::fd_count_cur;
    use perf_monitor::io::get_process_io_stats;
    use perf_monitor::mem::get_process_memory_info;

    // cpu
    let core_num = processor_numbers().unwrap();
    let mut stat_p = ProcessStat::cur().unwrap();
    let mut stat_t = ThreadStat::cur().unwrap();

    let _ = (0..1_000).into_iter().sum::<i128>();

    let usage_p = stat_p.cpu().unwrap() * 100f64;
    let usage_t = stat_t.cpu().unwrap() * 100f64;

    println!("[CPU] core Number: {}, process usage: {:.2}%, current thread usage: {:.2}%", core_num, usage_p, usage_t);

    // mem
    let mem_info = get_process_memory_info().unwrap();
    println!("[Memory] memory used: {} bytes, virtural memory used: {} bytes ", mem_info.resident_set_size, mem_info.virtual_memory_size);

    // fd
    let fd_num = fd_count_cur().unwrap();
    println!("[FD] fd number: {}", fd_num);

    // io
    let io_stat = get_process_io_stats().unwrap();   
    println!("[IO] io-in: {} bytes, io-out: {} bytes", io_stat.read_bytes, io_stat.write_bytes);
```

The above code should have the following output:
```txt
[CPU] core Number: 12, process usage: 502.16%, current thread usage: 2.91%
[Memory] memory used: 1073152 bytes, virtural memory used: 4405747712 bytes 
[FD] fd number: 7
[IO] io-in: 0 bytes, io-out: 32768 bytes
```

See [examples](./examples/activity_monitor.rs) for details. 

# Perfomance
We are concerned about the overhead associated with obtaining performance information. We try to use the most efficient methods while ensuring the API usability.

For example, CPU usage and FD number cost on these devices has following result:
- MacOS: MacBookPro15,1; 6-Core Intel Core i7; 2.6GHz; 16GB
- Windows: Windows10; Intel Core i3-2310M; 2.10GHz; 64bit; 4GB
- Andorid: Pixel 2; android 10

| profiling | Windows | MacOS | Android |
| :--- | :---: | :---: | :---: | 
| thread CPU usage (ms) | 3 | 0.45 | 16 |
| FD number (ms) | 0.15 | 0.07 | 10 |

# Supported Platform

| profiling | Windows | MacOS | iOS | Android | Linux |
| :--- | :---: | :---: | :---: | :---: | :---: |
| [CPU](https://docs.rs/perf_monitor/cpu/index.html) | ✅ | ✅ |✅ |✅ |✅ |
| [Memory](https://docs.rs/perf_monitor/mem/index.html) | ✅ |✅ |✅ |✅ |✅ |
| [FD count](https://docs.rs/perf_monitor/fd/index.html) | ✅ |✅ |❌ |✅ |✅ |
| [IO](https://docs.rs/perf_monitor/io/index.html) | ✅ |✅ |✅ |✅ |✅ 

See [documents](https://docs.rs/perf_monitor/) of each module for usage and more details.

# Rust Version

To compile document require the nightly version, others should work both in stable and nightly version.


```shell
cargo build

cargo +nightly doc 

cargo +nightly test
cargo test --lib
```

# Contribution

Contributions are welcome!

Open an issue or create a PR to report bugs, add new features or improve documents and tests.
If you are a new contributor, see [this page](https://github.com/firstcontributions/first-contributions) for help.


# Why perf-monitor-rs?

There are some crates to do similar things, such as [spork](https://github.com/azuqua/spork.rs), [procfs](https://github.com/eminence/procfs), and [sysinfo](https://github.com/GuillaumeGomez/sysinfo). 

Our application needs to monitor itself at runtime to help us find out performance issues. For example, when the CPU usage rises abnormally, we want to figure out which threads cause this. 

However, none of the above crates meet our needs. 

* `spork` can't get other thread information other than the calling thread. Only memory and CPU information can be processed. And it stops updating for years.
* `procfs` looks good enough now, but only support the Linux platform. In its early stages, when we developed perf_monitor_rs, there was no way to get thread information.
* `sysinfo` support all platform we need, but we think its interface is not elegant, because an explicit refresh is required before each call, otherwise an old value will be retrieved and you are not able to tell from the returning value. More importantly, it lacks some features like fd, CPU usage. 

If you are building a cross-platform application and facing the same problem, we hope perf_monitor_rs can be your first choice. 

# License
perf-monitor is providing under the MIT license. See [LICENSE](./LICENSE).
