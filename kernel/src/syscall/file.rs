// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! File-related syscalls

use crate::process::my_proc;
use crate::syscall::{arg_int, arg_uint, arg_ptr, arg_fd, arg_ptr_mut};
use crate::file::{File, Console, FsFile};
use alloc::sync::Arc;
use crate::spinlock::Mutex;
use crate::symbols::PAGE_SIZE;
use crate::virtio::BSIZE;

/// write syscall
pub fn sys_write() -> i32 {
    let p = my_proc();
    let sz = arg_uint(&p.trapframe, 2);
    if sz > BSIZE {
        panic!("size > BSIZE not supported");
    }
    let content = arg_ptr(&p.pgtable, &p.trapframe, 1, sz);
    let u8_slice = unsafe { core::slice::from_raw_parts(content, sz) };
    let file = arg_fd(&p, 0);
    match (*file).as_ref() {
        File::Device(dev) => dev.write(u8_slice),
        File::FsFile(file) => file.write(u8_slice),
        _ => { unimplemented!(); }
    }
}

/// read syscall
pub fn sys_read() -> i32 {
    let p = my_proc();
    let sz = arg_uint(&p.trapframe, 2);
    if sz > BSIZE {
        panic!("size > BSIZE not supported");
    }
    let content = arg_ptr_mut(&p.pgtable, &p.trapframe, 1, sz);
    let u8_slice = unsafe { core::slice::from_raw_parts_mut(content, sz) };
    let file = arg_fd(&p, 0);
    match (*file).as_ref() {
        File::Device(dev) => dev.read(u8_slice),
        File::FsFile(file) => file.read(u8_slice),
        _ => { unimplemented!(); }
    }
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
    let sz = arg_uint(&p.trapframe, 1);
    let mode = arg_uint(&p.trapframe, 2);
    let content = arg_ptr(&p.pgtable, &p.trapframe, 0, sz);
    let path = core::str::from_utf8(unsafe { core::slice::from_raw_parts(content, sz) }).unwrap();
    let fd = match next_available_fd(&p.files) {
        Some(fd) => fd,
        None => { return -1; }
    };
    if path == "/console" {
        p.files[fd] = Some(Arc::new(File::Device(box Console {})));
    } else {
        p.files[fd] = Some(Arc::new(File::FsFile(FsFile::open(path, mode))));
    }
    return fd as i32;
}

/// close syscall
pub fn sys_close() -> i32 {
    let p = my_proc();
    let fd = arg_int(&p.trapframe, 0) as usize;
    p.files[fd] = None;
    0
}

/// dup syscall
pub fn sys_dup() -> i32 {
    let p = my_proc();
    let old_fd = arg_uint(&p.trapframe, 0);
    let fd = match next_available_fd(&p.files) {
        Some(fd) => fd,
        None => { return -1; }
    };
    p.files[fd] = Some(p.files[old_fd].as_ref().unwrap().clone());
    use crate::info;
    fd as i32
}
