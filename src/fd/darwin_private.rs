//! This is assumed to be private api due to App Store's rejection: https://github.com/dotnet/maui/issues/3290
//!
//! Inspired by lsof: <https://github.com/apple-opensource/lsof/blob/da09c8c6436286e5bd8c400b42e86b54404f12a7/lsof/dialects/darwin/libproc/dproc.c#L623>

use libc::proc_taskallinfo;
use std::mem::MaybeUninit;
use std::os::raw::c_int;
use std::os::raw::c_void;

const PROC_PIDLISTFDS: c_int = 1;
const PROC_PIDTASKALLINFO: c_int = 2;

extern "C" {
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
}

#[repr(C)]
pub struct proc_fdinfo {
    pub proc_fd: i32,
    pub proc_fdtype: u32,
}

pub fn fd_count_cur() -> std::io::Result<usize> {
    fd_count_pid(std::process::id())
}

pub fn fd_count_pid(pid: u32) -> std::io::Result<usize> {
    let pid = pid as i32;
    let max_fds = unsafe {
        let mut info: MaybeUninit<proc_taskallinfo> = MaybeUninit::zeroed();
        let buffersize = std::mem::size_of::<proc_taskallinfo>() as c_int;
        let ret = proc_pidinfo(
            pid,
            PROC_PIDTASKALLINFO,
            0,
            info.as_mut_ptr() as _,
            buffersize,
        );
        if ret <= 0 {
            return Err(std::io::Error::from_raw_os_error(ret));
        }
        if ret < buffersize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "proc_pidinfo(PROC_PIDTASKALLINFO) too few bytes",
            ));
        }
        info.assume_init_ref().pbsd.pbi_nfiles as c_int
    };
    let buffersize = max_fds * std::mem::size_of::<proc_fdinfo>() as c_int;
    let mut buffer = vec![0u8; buffersize as usize];
    let ret = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDLISTFDS,
            0,
            buffer.as_mut_ptr() as _,
            buffersize,
        )
    };
    if ret <= 0 {
        Err(std::io::Error::from_raw_os_error(ret))
    } else {
        Ok(ret as usize / std::mem::size_of::<proc_fdinfo>())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fd_count_private() {
        // case1: open some files and do not close them.
        {
            let mut buf = vec![];
            const NUM: usize = 100;
            let init_count = fd_count_cur().unwrap();

            for i in 0..NUM {
                let fname = format!("/tmp/fd_count_test_tmpfile{}", i);
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(fname);
                buf.push(file);
            }
            let count = fd_count_cur().unwrap();
            assert_eq!(NUM + init_count, count);
        }

        // case2: compare the result with lsof.
        {
            let count_devfd = fd_count_cur().unwrap();
            let count_lsof = fd_lsof() - 2; // minus pipe fd between parent process and child process.
            assert_eq!(count_lsof, count_devfd);
        }
    }

    fn fd_lsof() -> usize {
        let pid = unsafe { libc::getpid() };
        let output = std::process::Command::new("lsof")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let output_txt = String::from_utf8(output.stdout).unwrap();
        let count_lsof = output_txt
            .lines()
            .filter(|s| s.find("cwd").is_none() && s.find("txt").is_none())
            .map(|s| println!("{}", s))
            .count();

        count_lsof - 1 // minus title line of lsof output.
    }
}
