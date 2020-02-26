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
use crate::spinlock::Mutex;
use crate::arch;
use alloc::boxed::Box;
use core::mem::MaybeUninit;
use core::borrow::BorrowMut;
use crate::arch::hart_id;

/// An array holding all CPU information
static mut CPUS: [CPU; NCPUS] = [CPU::zero(); NCPUS];

/// Enum describing a process in process pool
///
/// `NoProc`: No process associated with this pid
///
/// `Scheduled`: This process is scheduled on one CPU
/// 
/// `Pooling`: This process is not being scheduled
///
/// `BeingSlept`: This process holds a sleep lock and is to be put back
pub enum ProcInPool {
    NoProc,
    Scheduled,
    Pooling(Box<Process>),
    BeingSlept,
}

/// An array holding all process information.
/// 
/// # Examples
///
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
pub static PROCS_POOL: Mutex<[ProcInPool; NMAXPROCS]> = Mutex::new([ProcInPool::NoProc; NMAXPROCS], "proc pool");

pub unsafe fn init() {}

/// Get CPU object of current hart.
pub fn my_cpu() -> &'static mut CPU {
    unsafe { &mut CPUS[arch::hart_id()] }
}

/// Get reference to current running process on current hart.
///
/// Note that this reference is always valid no matter
/// which hart this process is scheduled.
/// On the contrary, `&mut my_cpu().process` can't
/// be moved between harts.
///
/// ## Examples
///
/// ```
/// use crate::{arch, process};
/// let p = process::my_proc();
/// arch::intr_on();
/// // There may be timer interrupt at any time, and context
/// // of this process in kernel may be scheduled to another
/// // hart.
/// p.trapframe.epc += 4;
/// ```
///
/// On the contrary, the following code may cause problems.
///
/// ```
/// use crate::{arch, process};
/// let p = &mut my_cpu().process.unwrap();
/// arch::intr_on();
/// // After this kernel context is scheduled on another
/// // hart, p is no longer valid, as previous hart may
/// // have scheduled a different process, or there is
/// // no process running at all.
/// p.trapframe.epc += 4; // Load page fault
/// ```
pub fn my_proc() -> &'static mut Process {
    let proc_cpu = my_cpu();
    proc_cpu.process.as_mut().unwrap()
}

use crate::println;

pub fn debug() {
    for i in 0..NCPUS {
        match unsafe { &CPUS[i].process } {
            Some(x) => { println!("{} running on hart {}", x.pid, i); }
            _ => {}
        }
    }
    let pool = unsafe { PROCS_POOL.get() };
    for i in 0..NMAXPROCS {
        match &pool[i] {
            ProcInPool::Pooling(x) => { println!("{} pooling with state {:?}", x.pid, x.state); }
            ProcInPool::BeingSlept => { println!("{} being slept", i); }
            _ => {}
        }
    }
}