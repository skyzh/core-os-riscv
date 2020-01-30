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
        if let (occupied, Some(_p)) = p {
            if !*occupied {
                continue;
            }
            if _p.state == ProcessState::RUNNABLE {
                let p = core::mem::replace(p, (true, None));
                return p.1;
            }
        }
    }
    None
}

pub fn put_back_proc(p: Box<Process>) {
    let mut pool = PROCS_POOL.lock();
    let p_in_pool = &mut pool[p.pid as usize];
    if p_in_pool.1.is_none() {
        *p_in_pool = (true, Some(p));
        return;
    }
    panic!("pid {} already occupied", p.pid);
}

pub fn scheduler() -> ! {
    let c = my_cpu();
    loop {
        arch::intr_on();
        if let Some(p) = find_next_runnable_proc() {
            c.process = Some(p);
            let p = c.process.as_mut().unwrap();
            // info!("scheduler: switching to {}", p.pid);
            p.state = ProcessState::RUNNING;
            let ctx = core::mem::replace(&mut p.context, box Context::zero());
            // info!("swtch to proc");
            swtch(&mut c.scheduler_context, *ctx);
            // info!("come back");
            let p = core::mem::replace(&mut c.process, None).unwrap();
            put_back_proc(p);
        }
    }
}
