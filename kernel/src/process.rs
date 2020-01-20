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

pub static CPUS: [Mutex<CPU>; NCPUS] = [Mutex::new(CPU::zero(), "cpu"); NCPUS];
pub static PROCS: [Mutex<Process>; NMAXPROCS] = [Mutex::new(Process::zero(), "proc"); NMAXPROCS];
