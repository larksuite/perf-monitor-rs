pub fn fd_count_cur() -> std::io::Result<usize> {
    // Remove the opening fd created by `read_dir`
    std::fs::read_dir("/dev/fd").map(|entries| entries.count().saturating_sub(1))
}

#[cfg(test)]
mod test {
    use super::*;

    // We put these test case in one test to make them get executed one by one.
    // Parallel execution causes them to interact with each other and fail the test.
    #[test]
    fn test_fd_count() {
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
