// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Internal representation of syscall
//!
//! As Rust primitives can't be directly transferred between
//! kernel space and user space, all Rust primitives will be
//! transmuted into pointers in `syscall` module, and then
//! this module will finally trap into kernel.

global_asm!(include_str!("usys.S"));

extern "C" {
    pub fn __write(fd: i32, content: *const u8, sz: i32) -> i32;
    pub fn __exit(code: i32) -> !;
    pub fn __fork() -> i32;
    pub fn __exec(path: *const u8, sz: i32, args: *const *const u8) -> !;
}
