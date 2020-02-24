// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! File-related syscalls

use crate::process::my_proc;
use crate::syscall::{argint, arguint, argptr, argfd, arg_ptr_mut};
use crate::file::{File, Console};
use alloc::sync::Arc;
use crate::spinlock::Mutex;

/// write syscall
pub fn sys_write() -> i32 {
    let mut p = my_proc();
    // TODO: sz is not always less than a page
    let sz = arguint(&p.trapframe, 2);
    let content = argptr(&p.pgtable, &p.trapframe, 1, sz);
    let u8_slice = unsafe { core::slice::from_raw_parts(content, sz) };
    let mut file = argfd(&mut p, 0).lock();
    file.write(u8_slice)
}

/// read syscall
pub fn sys_read() -> i32 {
    let mut p = my_proc();
    // TODO: sz is not always less than a page
    let sz = arguint(&p.trapframe, 2);
    let content = arg_ptr_mut(&p.pgtable, &p.trapframe, 1, sz);
    let u8_slice = unsafe { core::slice::from_raw_parts_mut(content, sz) };
    let mut file = argfd(&mut p, 0).lock();
    file.read(u8_slice)
}

/// find a available file descriptor from files array in process
fn next_available_fd<T>(files: &[Option<T>]) -> Option<usize> {
    for i in 0..files.len() {
        match files[i] {
            None => { return Some(i); }
            _ => { continue; }
        }
    }
    return None;
}

/// open syscall, currently only support `/console` file.
pub fn sys_open() -> i32 {
    let p = my_proc();
    let sz = arguint(&p.trapframe, 1);
    let content = argptr(&p.pgtable, &p.trapframe, 0, sz);
    let path = core::str::from_utf8(unsafe { core::slice::from_raw_parts(content, sz) }).unwrap();
    let fd = match next_available_fd(&p.files) {
        Some(fd) => fd,
        None => { return -1; }
    };
    if path == "/console" {
        p.files[fd] = Some(Arc::new(Mutex::new(Console {}, "file lock")))
    } else {
        panic!("unsupported file: {}", path);
    }
    return fd as i32;
}

/// close syscall
pub fn sys_close() -> i32 {
    unimplemented!()
}

/// dup syscall
pub fn sys_dup() -> i32 {
    let p = my_proc();
    let old_fd = arguint(&p.trapframe, 0);
    let fd = match next_available_fd(&p.files) {
        Some(fd) => fd,
        None => { return -1; }
    };
    p.files[fd] = Some(p.files[old_fd].as_ref().unwrap().clone());
    use crate::info;
    fd as i32
}
