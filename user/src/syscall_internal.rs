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
    pub fn __read(fd: i32, content: *mut u8, sz: i32) -> i32;
    pub fn __exit(code: i32) -> !;
    pub fn __fork() -> i32;
    pub fn __exec(path: *const u8, path_sz: i32, arg_cnt: i32, args: *const *const u8, args_sz: *const i32) -> !;
    pub fn __open(path: *const u8, sz: i32, mode: i32) -> i32;
    pub fn __close(fd: i32) -> i32;
    pub fn __dup(fd: i32) -> i32;
    pub fn __wait(pid: i32) -> i32;
}
