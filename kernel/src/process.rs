mod trapframe;
pub use trapframe::*;

mod cpu;
pub use cpu::*;

mod process;
pub use process::*;

mod context;
pub use context::*;

use crate::symbols::NCPUS;
use crate::nulllock::Mutex;

pub static CPUS: [Mutex<CPU>; NCPUS] = [Mutex::new(CPU::zero()); NCPUS];
