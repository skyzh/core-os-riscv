// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::wait_forever;
use crate::arch;
use crate::trap::usertrapret;
use crate::symbols::*;
use crate::process::{PROCS_POOL, ProcessState, swtch, Register, Context, my_cpu, Process};
use crate::info;
use crate::panic;
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use core::mem::MaybeUninit;

fn find_next_runnable_proc() -> Option<Box<Process>> {
    let mut pool = PROCS_POOL.lock();
    for pid in 0..NMAXPROCS {
        let p = &mut pool[pid];
        if p.0 {
            let _p = unsafe { p.1.get_mut() };
            if _p.state == ProcessState::RUNNABLE {
                drop(_p);
                let p = core::mem::replace(p, (false, MaybeUninit::uninit()));
                return Some(unsafe { p.1.assume_init() });
            }
        }
    }
    None
}

pub fn put_back_proc(p: Box<Process>) {
    let mut pool = PROCS_POOL.lock();
    for pid in 0..NMAXPROCS {
        let p_in_pool = &mut pool[pid];
        if !p_in_pool.0 {
            *p_in_pool = (true, MaybeUninit::new(p));
            return;
        }
    }
    panic!("no available pool");
}

pub fn scheduler() -> ! {
    let c = my_cpu();
    loop {
        arch::intr_on();
        if let Some(p) = find_next_runnable_proc() {
            c.process = MaybeUninit::new(p);
            let p = unsafe { c.process.get_mut() };
            info!("scheduler: switching to {}", p.pid);
            p.state = ProcessState::RUNNING;
            let ctx = core::mem::replace(&mut p.context, box Context::zero());
            info!("swtch to proc");
            swtch(&mut c.scheduler_context, *ctx);
            info!("come back");
            let p = core::mem::replace(&mut c.process, MaybeUninit::uninit());
            put_back_proc(unsafe { p.assume_init() });
        }
    }
}
