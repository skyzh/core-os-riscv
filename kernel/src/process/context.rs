// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![allow(non_camel_case_types)]

use crate::{println, panic, info};
use crate::process::{my_cpu, my_proc, Process, ProcessState};
use crate::spinlock::MutexGuard;

/// Kernel context
#[repr(C)]
pub struct Context {
    // ra + sp + s0 ~ s11
    pub regs: [usize; 14]
}

impl Context {
    pub const fn zero() -> Self {
        Self {
            regs: [0; 14]
        }
    }
}

pub enum ContextRegisters {
    ra = 0,
    sp = 1
}

extern "C" {
    /// __swtch in `swtch.S`
    fn __swtch(current: &mut Context, to: &Context);
}

/// switch context
pub fn swtch(current: &mut Context, next: Context) {
    unsafe { __swtch(current, &next); }
}

/// switch to scheduler context
pub fn sched() {
    let c = my_cpu();
    let p = my_proc();

    if p.state == ProcessState::RUNNING {
        panic!("process running");
    }

    if crate::arch::intr_get() {
        panic!("yield interruptable");
    }

    let ctx = core::mem::replace(&mut c.scheduler_context, Context::zero());

    let intena = c.intr_lock.is_enabled_before;

    swtch(&mut p.context, ctx);
    c.intr_lock.is_enabled_before = intena;
}

/// give up CPU to scheduler
pub fn yield_cpu() {
    let p = my_proc();
    p.state = ProcessState::RUNNABLE;
    sched();
}
