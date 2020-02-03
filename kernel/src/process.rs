// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Process and scheduling
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
use core::borrow::BorrowMut;

/// An array holding all CPU information
static mut CPUS: [CPU; NCPUS] = [CPU::zero(); NCPUS];

/// An array holding all process information.
/// 
/// # Examples
/// ```
/// let pool = PROCS_POOL.lock();
/// let p = &mut pool[0];
/// if p.0 == false {
///     println!("This pid is not occupied by any process.");
/// }
/// if p.0 == true {
///     if p.1.is_none() {
///         println!("The process is running on one CPU.");
///     } else {
///         println!("The process is to be scheduled.");
///         assert_eq!(p.1.pid, 0);  // process of PID x is stored at PROCS_POOL[x]
///     }
/// }
/// ```
pub static PROCS_POOL: Mutex<[(bool, Option<Box<Process>>); NMAXPROCS]> = Mutex::new([(false, None); NMAXPROCS]);

/// Get CPU object of current hart
pub fn my_cpu() -> &'static mut CPU {
    unsafe { &mut CPUS[arch::hart_id()] }
}

/// Get current running process on current hart
pub fn my_proc() -> &'static mut Box<Process> {
    let proc_cpu = my_cpu();
    proc_cpu.process.as_mut().unwrap()
}
