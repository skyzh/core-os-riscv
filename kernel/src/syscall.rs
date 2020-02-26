// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Module for processing syscall
//! 
//! All functions that begin with `sys_` will extract
//! parameters from trap frame, turn pointers into
//! Rust primitives and call corresponding functions
//! with these parameters in kernel code.
//!
//! For specifications and how to do syscalls, refer to
//! [syscall module in user crate](../../user/syscall/index.html).

mod gen;
mod file;

pub use gen::*;
use crate::process::{TrapFrame, Register, my_proc, fork, exec, exit, Process};
use crate::{info, panic, print, println};
use crate::page;
use crate::mem::{align_val, page_down};
use crate::symbols::{PAGE_ORDER, PAGE_SIZE};
use file::*;
use alloc::sync::Arc;
use crate::file::File;
use alloc::boxed::Box;
use crate::spinlock::Mutex;

/// Get the `pos`th argument from syscall
pub fn argraw(tf: &TrapFrame, pos: usize) -> usize {
    match pos {
        0 => tf.regs[Register::a0 as usize],
        1 => tf.regs[Register::a1 as usize],
        2 => tf.regs[Register::a2 as usize],
        3 => tf.regs[Register::a3 as usize],
        4 => tf.regs[Register::a4 as usize],
        5 => tf.regs[Register::a5 as usize],
        _ => unreachable!()
    }
}

/// Get the `pos`th argument as i32 from syscall
pub fn arg_int(tf: &TrapFrame, pos: usize) -> i32 {
    argraw(tf, pos) as i32
}

/// Get the `pos`th argument as usize from syscall
pub fn arg_uint(tf: &TrapFrame, pos: usize) -> usize {
    let sz = argraw(tf, pos) as i32;
    if sz < 0 {
        panic!("invalid unsigned int");
    }
    sz as usize
}

/// Get the `pos`th argument as a pointer from syscall, return kernel-space pointer (involve security issues!)
pub fn arg_ptr(pgtable: &page::Table, tf: &TrapFrame, pos: usize, sz: usize) -> *const u8 {
    let ptr = argraw(tf, pos);
    let pg_begin = page_down(ptr);
    if ptr + sz >= pg_begin + PAGE_SIZE {
        panic!("out of bound!");
    }
    let paddr = pgtable.paddr_of(pg_begin).unwrap();
    unsafe { (paddr as *const u8).add(ptr - pg_begin) }
}

/// Get the `pos`th argument as a pointer from syscall, return kernel-space mutable pointer (involve security issues!)
pub fn arg_ptr_mut(pgtable: &page::Table, tf: &TrapFrame, pos: usize, sz: usize) -> *mut u8 {
    arg_ptr(pgtable, tf, pos, sz) as *mut u8
}


/// Get file corresponding to a file descriptor
pub fn arg_fd(p: &Process, pos: usize) -> &Arc<File> {
    let fd = argraw(&p.trapframe, pos);
    match &p.files[fd] {
        Some(x) => return x,
        None => panic!("invalid file handler {}", fd)
    }
}

/// fork syscall entry
fn sys_fork() -> i32 {
    fork()
}

/// exec syscall entry
fn sys_exec() -> i32 {
    let path;
    {
        let p = my_proc();
        let sz = arg_uint(&p.trapframe, 1);
        let ptr = arg_ptr(&p.pgtable, &p.trapframe, 0, sz);
        path = unsafe {
            // First, we build a &[u8]...
            let slice = core::slice::from_raw_parts(ptr, sz);
            // ... and then convert that slice into a string slice
            core::str::from_utf8(slice).unwrap()
        };
    }
    if path == "/init" {
        info!("running tests before init...");
        crate::test::run_tests();
    }
    exec(path);
    0
}

/// exit syscall entry
fn sys_exit() -> i32 {
    let code;
    {
        let p = my_proc();
        code = arg_int(&p.trapframe, 0);
    }
    exit(code);
}

/// Process all syscall
pub fn syscall() -> i32 {
    let syscall_id;
    {
        let p = my_proc();
        let tf = &p.trapframe;
        syscall_id = tf.regs[Register::a7 as usize] as i64;
    }
    match syscall_id {
        SYS_WRITE => sys_write(),
        SYS_READ => sys_read(),
        SYS_FORK => sys_fork(),
        SYS_EXEC => sys_exec(),
        SYS_EXIT => sys_exit(),
        SYS_DUP => sys_dup(),
        SYS_OPEN => sys_open(),
        SYS_CLOSE => sys_close(),
        _ => unreachable!()
    }
}
