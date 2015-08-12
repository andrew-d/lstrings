use std::fs;
use std::convert::AsRef;
use std::path::Path;
use std::slice;

use ::libc;
use mmap::{MemoryMap, MapOption};


#[cfg(unix)]
fn get_fd(file: &fs::File) -> libc::c_int {
    use std::os::unix::io::AsRawFd;
    file.as_raw_fd()
}

#[cfg(windows)]
fn get_fd(file: &fs::File) -> libc::HANDLE {
    use std::os::windows::io::AsRawHandle;
    file.as_raw_handle() as libc::HANDLE
}

pub fn with_file_mmap<P, F, T>(path: P, f: F) -> T
where P: AsRef<Path>,
      F: Fn(&[u8]) -> T
{
    let file = fs::OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap();

    // Get the size of the file.
    let len = file.metadata().unwrap().len() as usize;

    let fd = get_fd(&file);

    let chunk = MemoryMap::new(len, &[
       MapOption::MapReadable,
       MapOption::MapFd(fd),
    ]).unwrap();

    let file_data: &[u8] = unsafe {
        slice::from_raw_parts(chunk.data() as *const _, chunk.len())
    };

    f(file_data)
}
