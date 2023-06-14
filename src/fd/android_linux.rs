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
        let mut buf = vec![];
        const NUM: usize = 100;

        // open some files and do not close them.
        for i in 0..NUM {
            let fname = format!("/tmp/tmpfile{}", i);
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(fname);
            buf.push(file);
        }
        let count = fd_count_cur().unwrap();

        assert!(NUM + 3 <= count);

        // close files
        while let Some(_) = buf.pop() {}
        let count = fd_count_cur().unwrap();
        assert!(3 <= count);
        assert!(NUM > count);
    }
}
