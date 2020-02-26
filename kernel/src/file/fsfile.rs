// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! File on file system

use crate::virtio::{VIRTIO, VirtIO, BSIZE};
use crate::{print, println};
use crate::spinlock::Mutex;

pub struct FsFile {
    offset: usize,
    sz: usize,
    rw_offset: Mutex<(usize, usize)>,
    readable: bool,
    writable: bool,
}

const HEADER_SIZE: usize = 1024;
const FILE_MAX: usize = 1024;

impl FsFile {
    fn get_file_info(virtio: &mut VirtIO, path: &str) -> Option<(usize, usize)> {
        for id in 0..FILE_MAX {
            let b = virtio.read(1, id as u32);
            let sz = unsafe { core::ptr::read(b.data.as_ptr() as *const usize) };
            let offset = unsafe { core::ptr::read((b.data.as_ptr() as *const usize).add(1)) };
            if sz == 0 {
                break;
            }
            let name_sz = {
                let mut i = 16;
                loop {
                    let d = b.data[i];
                    if d == 0 {
                        break;
                    }
                    i += 1;
                    if i == b.data.len() { break; }
                }
                i - 16
            };
            let u8_slice = unsafe { core::slice::from_raw_parts(b.data.as_ptr().add(16), name_sz) };
            let name = core::str::from_utf8(u8_slice).unwrap();
            if name == path {
                return Some((offset, sz));
            }
        }
        None
    }

    pub fn open(path: &str, mode: usize) -> Self {
        let virtio = VIRTIO();
        let (offset, sz) = match Self::get_file_info(virtio, path) {
            Some(x) => x,
            None => { panic!("{} not found", path); }
        };
        Self {
            offset,
            sz,
            rw_offset: Mutex::new((0, 0), "file rw offset"),
            readable: true,
            writable: true,
        }
    }

    pub fn read(&self, content: &mut [u8]) -> i32 {
        use crate::info;
        if !self.readable { return -1; }
        let virtio = VIRTIO();
        let read_offset = self.rw_offset.lock().0;
        let read_sz = (self.sz - read_offset).min(content.len());
        let result = virtio.read(1, ((self.offset + read_offset) / BSIZE) as u32);
        // content.copy_from_slice(&result.data[0..content.len()]);
        for i in 0..read_sz {
            content[i] = result.data[i];
        }
        self.rw_offset.lock().0 = read_offset + read_sz;
        // println!("off: {:x}, rsz: {:x}, sz: {:x}", read_offset, read_sz, self.sz);
        return read_sz as i32;
    }

    pub fn write(&self, content: &[u8]) -> i32 {
        if !self.writable { return -1; }
        unimplemented!()
    }
}

pub mod tests {
    use super::*;

    pub fn tests() -> &'static [(&'static str, fn())] {
        &[
            ("open", test_open),
            ("read", test_read),
            ("read_elf", test_read_elf),
        ]
    }

    use crate::{print, println};

    /// Test open
    pub fn test_open() {
        let f = FsFile::open("/test.txt", 0);
    }

    /// Test read
    pub fn test_read() {
        let f = FsFile::open("/test.txt", 0);
        let mut content = [0; 10];
        assert_eq!(f.read(&mut content), 10);
        assert_eq!(content, [48, 49, 50, 51, 52, 53, 54, 55, 56, 57]);
    }

    /// Test read
    pub fn test_read_elf() {
        let f = FsFile::open("/test1", 0);
        let mut content = [0; 1024];
        while f.read(&mut content) == 1024 {}
    }
}

