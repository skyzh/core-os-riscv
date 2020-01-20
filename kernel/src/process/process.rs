use super::{TrapFrame, Context, Register};
use crate::page;
use crate::symbols::*;
use crate::alloc;
use crate::cpu;
use crate::println;

pub enum ProcessState {
    UNUSED, SLEEPING, RUNNABLE, RUNNING, ZOMBIE
}

#[repr(C)]
#[repr(align(4096))]
pub struct Process {
    pub pgtable: page::Table,
    pub trapframe: TrapFrame,
    pub context: Context,
    pub state: ProcessState
}

impl Process {
    pub const fn zero() -> Self {
        Self {
            trapframe: TrapFrame::zero(),
            context: Context::zero(),
            pgtable: page::Table::new(),
            state: ProcessState::UNUSED
        }
    }

    pub fn init(pid: i64) {
        if pid < 0 {
            panic!("invalid pid");
        }
        let mut process = crate::process::PROCS[pid as usize].lock();
        *process = Self {
            trapframe: TrapFrame::zero(),
            context: Context::zero(),
            pgtable: page::Table::new(),
            state: ProcessState::UNUSED
        };

        // map trampoline
        process.pgtable.map(
            TRAMPOLINE_START,
            unsafe { TRAMPOLINE_TEXT_START },
            page::EntryAttributes::RX as usize,
            0,
        );
        let trapframe = &process.trapframe as *const _ as usize;
        // map trapframe
        process.pgtable.map(
            TRAPFRAME_START,
            trapframe,
            page::EntryAttributes::RW as usize,
            0,
        );
    }
}

pub fn init_proc() {
    Process::init(0);
    let mut process = crate::process::PROCS[0].lock();
    let entry = crate::elf::parse_elf(
        include_bytes!("../../../target/riscv64gc-unknown-none-elf/release/init"),
        &mut process.pgtable
    );
    // map user stack
    let seg = alloc::ALLOC().lock().allocate(alloc::PAGE_SIZE);
    let stack_begin = 0x80001000;
    process.pgtable.map(
        stack_begin,
        seg as usize,
        page::EntryAttributes::URW as usize,
        0,
    );
    process.trapframe.epc = entry as usize;
    process.trapframe.regs[Register::sp as usize] = stack_begin + 0x1000; // sp
    process.state = ProcessState::RUNNABLE;
    // process.pgtable.walk();
}
