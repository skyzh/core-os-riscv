// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! File on file system

use crate::virtio::{VIRTIO, VirtIO, BSIZE};
use crate::{print, println};

pub struct FsFile {
    offset: usize,
    sz: usize,
    read_offset: usize,
    write_offset: usize,
    readable: bool,
    writable: bool
}

const HEADER_SIZE: usize = 1024;
const FILE_MAX: usize = 1024;

impl FsFile {
    fn get_file_info(virtio: &mut VirtIO, path: &str) -> Option<(usize, usize)> {
        for id in 0..FILE_MAX {
            let b = virtio.read(1, id as u32);
            let sz = unsafe {core::ptr::read(b.data.as_ptr() as *const usize) };
            let offset = unsafe {core::ptr::read((b.data.as_ptr() as *const usize).add(1)) };
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
        let (offset, sz) = Self::get_file_info(virtio, path).unwrap();
        Self {
            offset, sz,
            read_offset: 0,
            write_offset: 0,
            readable: true,
            writable: true
        }
    }

    pub fn read(&self, content: &mut [u8]) -> i32 {
        if !self.readable { return -1; }
        let virtio = VIRTIO();
        let result = virtio.read(1, ((self.offset + self.read_offset) / BSIZE) as u32);
        content.copy_from_slice(&result.data[0..content.len()]);
        return content.len() as i32;
    }

    pub fn write(&self, content: &[u8]) -> i32 {
        if !self.writable { return -1; }
        unimplemented!()
    }
}
