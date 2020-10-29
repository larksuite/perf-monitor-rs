// There is no api to retrieve the fd count of the process for ios.
// following links contains a available way, but it's complicated and
// inefficient. <https://stackoverflow.com/questions/4083608/on-ios-iphone-too-many-open-files-need-to-list-open-files-like-lsof>

pub type Result<T> = std::result::Result<T, ()>;

pub fn fd_count_cur() -> Result<usize> {
    unimplemented!()
}

pub fn fd_count_pid(pid: u32) -> Result<usize> {
    unimplemented!()
}
