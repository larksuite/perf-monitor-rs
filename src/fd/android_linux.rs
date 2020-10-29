pub type Result<T> = std::io::Result<T>;

pub fn fd_count_pid(pid: u32) -> Result<usize> {
    let path = format!("/proc/{}/fd", pid);
    let dir_entries = std::fs::read_dir(path)?;
    let count = dir_entries.count();
    Ok(count)
}

pub fn fd_count_cur() -> Result<usize> {
    let path = "/proc/self/fd";
    let dir_entries = std::fs::read_dir(path)?;
    let count = dir_entries.count();
    Ok(count)
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
