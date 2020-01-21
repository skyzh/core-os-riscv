// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

mod trapframe;
pub use trapframe::*;

mod cpu;
pub use cpu::*;

mod process;
pub use process::*;

mod context;
pub use context::*;

mod schedule;
pub use schedule::*;

use crate::symbols::*;
use crate::nulllock::Mutex;
use crate::arch;
use alloc::boxed::Box;
use core::mem::MaybeUninit;

static mut CPUS: [CPU; NCPUS] = [CPU::zero(); NCPUS];
pub static PROCS_POOL: Mutex<[(bool, MaybeUninit<Box<Process>>); NMAXPROCS]> = Mutex::new([(false, MaybeUninit::uninit()); NMAXPROCS], "proc");

pub fn my_cpu() -> &'static mut CPU {
    unsafe { &mut CPUS[arch::hart_id()] }
}

pub fn my_proc() -> &'static mut Box<Process> {
    let proc_cpu = my_cpu();
    unsafe { proc_cpu.process.get_mut() }
}
