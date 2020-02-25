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

use crate::syscall_internal::*;
use core::ptr::null;

/// Exit current process with exit code `code`.
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

pub const EXEC_MAX_ARGS: usize = 10;

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
    let arg_cnt = args.len();
    let mut args_sz = [0; EXEC_MAX_ARGS];
    let mut args_ptr = [null(); EXEC_MAX_ARGS];
    for i in 0..args.len() {
        args_sz[i] = args[i].len() as i32;
        args_ptr[i] = args[i].as_bytes().as_ptr() as *const u8;
    }
    unsafe {
        __exec(
            path.as_bytes().as_ptr() as *const u8,
            path.len() as i32,
            arg_cnt as i32,
            args_ptr.as_ptr(),
            args_sz.as_ptr()
        )
    }
}

/// Write `content` to file descriptor `fd`.
///
/// Returns number of characters written. A negative return value means error while writing.
///
/// # Examples
/// ```
/// use user::syscall::write;
/// use user::constant::STDOUT;
/// write(STDOUT, "Hello, World!");
/// ```
pub fn write(fd: i32, content: &[u8]) -> i32 {
    unsafe {
        __write(fd,
                content.as_ptr(),
                content.len() as i32)
    }
}

/// Read `content` from file descriptor `fd`.
///
/// You may read a maximum of `content.len()` characters from `fd`.
/// Returns number of characters read.
pub fn read(fd: i32, content: &mut [u8]) -> i32 {
    unsafe {
        __read(fd,
                content.as_mut_ptr(),
                content.len() as i32)
    }
}

/// Open file of `path` with `mode`.
///
/// This function returns file descriptor. Negative value means error.
///
/// # Examples
/// ```
/// use user::syscall::open;
/// let fd = open("/console", 0);
/// ```
pub fn open(path: &str, mode: i32) -> i32 {
    unsafe {
        __open(path.as_ptr(), path.len() as i32, mode)
    }
}

/// Close a file with file descriptor `fd`.
///
/// # Examples
/// ```
/// use user::syscall::close;
/// close(0);
/// ```
pub fn close(fd: i32) -> i32 {
    unsafe { __close(fd) }
}

/// Duplicate file descriptor `fd`.
///
/// Returns new file descriptor.
///
/// # Examples
/// ```
/// use user::syscall::dup;
/// use user::constant::STDIN;
/// let fd = dup(STDIN);
/// ```
pub fn dup(fd: i32) -> i32 {
    unsafe { __dup(fd) }
}

pub fn wait(pid: i32) -> i32 {
    unsafe { __wait(pid) }
}
