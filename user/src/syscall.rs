// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::syscall_internal::{__exit, __fork, __write};

pub fn exit(code: i32) -> ! {
    unsafe { __exit(code) }
}

pub fn fork() -> i32 {
    unsafe { __fork() }
}

pub fn write(fd: i32, content: &str) -> i32 {
    unsafe {
        __write(fd,
                content.as_bytes().as_ptr() as *const u8,
                content.len() as i32)
    }
}
