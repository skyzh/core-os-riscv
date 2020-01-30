// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::{TrapFrame, Context, Register, ContextRegisters};
use crate::{page, panic, info};
use crate::symbols::*;
use crate::mem;
use crate::arch;
use crate::println;
use crate::trap::usertrapret;
use alloc::boxed::Box;
use crate::process::{put_back_proc, my_proc, PROCS_POOL};
use crate::page::{Page, Table};
use crate::process::Register::a0;

#[derive(PartialEq)]
pub enum ProcessState {
    UNUSED, SLEEPING, RUNNABLE, RUNNING, ZOMBIE
}

#[repr(C)]
#[repr(align(4096))]
pub struct Process {
    pub pgtable: Box<page::Table>,
    pub trapframe: Box<TrapFrame>,
    pub context: Box<Context>,
    pub state: ProcessState,
    pub kstack: usize,
    pub pid: i32
}

impl Process {
    pub fn new(pid: i32) -> Self {
        Self::from_exist(pid, box page::Table::new(), box TrapFrame::zero())
    }

    pub fn from_exist(pid: i32, pgtable: Box<Table>, trapframe: Box<TrapFrame>) -> Self {
        if pid < 0 {
            panic!("invalid pid");
        }

        let kstack = Page::new();

        let mut p = Self {
            trapframe,
            pgtable,
            context: box Context::zero(),
            state: ProcessState::UNUSED,
            kstack: Box::into_raw(kstack) as usize,
            pid
        };

        // map trampoline
        p.pgtable.kernel_map(
            TRAMPOLINE_START,
            TRAMPOLINE_TEXT_START(),
            page::EntryAttributes::RX as usize
        );

        let trapframe = &*p.trapframe as *const _ as usize;
        // map trapframe
        p.pgtable.kernel_map(
            TRAPFRAME_START,
            trapframe,
            page::EntryAttributes::RW as usize
        );
        p.context.regs[ContextRegisters::ra as usize] = forkret as usize;
        p.context.regs[ContextRegisters::sp as usize] = p.kstack + PAGE_SIZE;

        p
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let kstack = unsafe { Box::from_raw(self.kstack as *mut Page) };
    }
}

#[no_mangle]
pub extern "C" fn forkret() {
    usertrapret();
}

pub fn init_proc() {
    let mut p = Process::new(0);
    let entry = crate::elf::parse_elf(
        include_bytes!("../../../target/riscv64gc-unknown-none-elf/release/init"),
        &mut p.pgtable
    );
    // map user stack
    let stack = page::Page::new();
    let stack_begin = 0x80001000;
    p.pgtable.map(
        stack_begin,
        stack,
        page::EntryAttributes::URW as usize
    );
    p.trapframe.epc = entry as usize;
    p.trapframe.regs[Register::sp as usize] = stack_begin + 0x1000; // sp
    p.state = ProcessState::RUNNABLE;
    put_back_proc(box p);
}

pub fn find_available_pid() -> Option<i32> {
    let pool = PROCS_POOL.lock();
    for i in 0..NMAXPROCS {
        if pool[i].0 == false {
            return Some(i as i32);
        }
    }
    None
}

pub fn fork() -> i32 {
    let p = my_proc();
    let f_pid = find_available_pid();
    if f_pid.is_none() {
        panic!("pid unavailable");
    }
    let f_pid = f_pid.unwrap();
    let pgtable = p.pgtable.clone();
    let trapframe = box *p.trapframe.clone();
    let mut fork_p = Process::from_exist(f_pid, pgtable, trapframe);
    fork_p.trapframe.regs[a0 as usize] = 0;
    fork_p.state = ProcessState::RUNNABLE;
    put_back_proc(box fork_p);
    f_pid
}
