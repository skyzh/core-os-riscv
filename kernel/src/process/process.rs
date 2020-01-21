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
use crate::process::put_back_proc;

#[derive(PartialEq)]
pub enum ProcessState {
    UNUSED, SLEEPING, RUNNABLE, RUNNING, ZOMBIE
}

#[repr(C)]
#[repr(align(4096))]
pub struct Process {
    pub pgtable: page::Table,
    pub trapframe: Box<TrapFrame>,
    pub context: Box<Context>,
    pub state: ProcessState,
    pub kstack: usize,
    pub pid: i64
}

impl Process {
    pub fn new(pid: i64) -> Self {
        if pid < 0 {
            panic!("invalid pid");
        }
        let kstack = mem::ALLOC().lock().allocate(1);
        let mut p = Self {
            trapframe: box TrapFrame::zero(),
            context: box Context::zero(),
            pgtable: page::Table::new(),
            state: ProcessState::UNUSED,
            kstack: kstack as usize,
            pid
        };

        // map trampoline
        p.pgtable.map(
            TRAMPOLINE_START,
            unsafe { TRAMPOLINE_TEXT_START },
            page::EntryAttributes::RX as usize,
            0,
        );

        let trapframe = &*p.trapframe as *const _ as usize;
        // map trapframe
        p.pgtable.map(
            TRAPFRAME_START,
            trapframe,
            page::EntryAttributes::RW as usize,
            0,
        );
        p.context.regs[ContextRegisters::ra as usize] = forkret as usize;
        p.context.regs[ContextRegisters::sp as usize] = p.kstack + PAGE_SIZE;

        p
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
    let seg = mem::ALLOC().lock().allocate(mem::PAGE_SIZE);
    let stack_begin = 0x80001000;
    p.pgtable.map(
        stack_begin,
        seg as usize,
        page::EntryAttributes::URW as usize,
        0,
    );
    p.trapframe.epc = entry as usize;
    p.trapframe.regs[Register::sp as usize] = stack_begin + 0x1000; // sp
    p.state = ProcessState::RUNNABLE;
    put_back_proc(box p);
}
