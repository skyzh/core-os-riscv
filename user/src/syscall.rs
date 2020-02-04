// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! All syscalls of core-os
//! 
//! Syscalls of core-os are defined and implemented with Rust
//! primitives (e.g. `str`, `[u8]`). This module will transmute
//! these Rust primitives into pointers and other machine-specific
//! representations before calling functions in `syscall_internal` and
//! trapping into kernel.
//!
//! Usage of syscalls is listed in their corresponding sub-page.

use crate::syscall_internal::{__exit, __fork, __write, __exec};

/// Exit current process with exit code `code`
/// 
/// # Examples
///
/// ```
/// use user::syscall::exit;
/// exit(0);
/// ```
pub fn exit(code: i32) -> ! {
    unsafe { __exit(code) }
}

/// Fork current process. 
/// 
/// Child process will get return value of 0.
/// Parent process (the one calling `fork`) will
/// get pid of child process.
/// 
/// # Examples
///
/// ```
/// use user::syscall::fork;
/// if fork() == 0 {
///     println!("subprocess!");
/// } else {
///     println!("parent process");
/// }
/// ```
pub fn fork() -> i32 {
    unsafe { __fork() }
}

/// Replace current process image with the new one
/// in the filesystem.
///
/// This function will not return.
///
/// # Examples
/// ```
/// use user::syscall::exec;
/// exec("/init", &[]);
/// ```
pub fn exec(path: &str, args: &[&str]) -> ! {
    unsafe { __exec(path.as_bytes().as_ptr() as *const u8, path.len() as i32, core::ptr::null()) }
}

pub fn write(fd: i32, content: &str) -> i32 {
    unsafe {
        __write(fd,
                content.as_bytes().as_ptr() as *const u8,
                content.len() as i32)
    }
}
