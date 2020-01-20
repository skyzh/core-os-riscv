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

pub static CPUS: [Mutex<CPU>; NCPUS] = [Mutex::new(CPU::zero()); NCPUS];
pub static PROCS: [Mutex<Process>; NMAXPROCS] = [Mutex::new(Process::zero()); NMAXPROCS];
