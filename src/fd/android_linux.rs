pub fn fd_count_pid(pid: u32) -> std::io::Result<usize> {
    // Subtract 2 to exclude `.`, `..` entries
    std::fs::read_dir(format!("/proc/{}/fd", pid)).map(|entries| entries.count().saturating_sub(2))
}

pub fn fd_count_cur() -> std::io::Result<usize> {
    // Subtract 3 to exclude `.`, `..` entries and fd created by `read_dir`
    std::fs::read_dir("/proc/self/fd").map(|entries| entries.count().saturating_sub(3))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fd_count() {
        #[cfg(target_os = "linux")]
        const TEMP_DIR: &str = "/tmp";
        #[cfg(target_os = "android")]
        const TEMP_DIR: &str = "/data/local/tmp";

        const NUM: usize = 100;

        // open some files and do not close them.
        let fds: Vec<_> = (0..NUM)
            .map(|i| {
                let fname = format!("{}/tmpfile{}", TEMP_DIR, i);
                std::fs::File::create(fname).unwrap()
            })
            .collect();
        let count = fd_count_cur().unwrap();

        dbg!(count);
        assert!(count >= NUM);
        let old_count = count;

        drop(fds);
        let count = fd_count_cur().unwrap();
        // Though tests are run in multi-thread mode without using nextest, we
        // assume NUM is big enough to make fd count lower in a short period.
        assert!(count < old_count);
    }
}
