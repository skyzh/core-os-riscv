// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::arch::wait_forever;
use crate::arch;
use crate::trap::usertrapret;
use crate::symbols::*;
use crate::process::{PROCS_POOL, ProcessState, swtch, Register, Context, my_cpu, Process};
use crate::{info, println};
use crate::panic;
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use crate::jump::*;

/// Find a runnable process whose pid >= `from_pid`
fn find_next_runnable_proc(from_pid: usize) -> Option<Box<Process>> {
    let mut pool = PROCS_POOL.lock();
    for pid in from_pid..NMAXPROCS {
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

/// Put process back to `PROCS_POOL`
pub fn put_back_proc(p: Box<Process>) {
    let mut pool = PROCS_POOL.lock();
    let p_in_pool = &mut pool[p.pid as usize];
    if p_in_pool.1.is_none() {
        *p_in_pool = (true, Some(p));
        return;
    }
    panic!("pid {} already occupied", p.pid);
}

/// Kernel scheduler
pub fn scheduler() -> ! {
    let c = my_cpu();
    let mut lst_pid = 0;
//    info!("scheduling on {}", arch::hart_id());
    loop {
        arch::intr_on();
        if let Some(p) = find_next_runnable_proc(lst_pid) {
            c.process = Some(p);
            let p = c.process.as_mut().unwrap();
            p.state = ProcessState::RUNNING;
            let ctx = core::mem::replace(&mut p.context, box Context::zero());
//            info!("scheduler {}: switching to {}", arch::hart_id(), p.pid);
            swtch(&mut c.scheduler_context, *ctx);
//            info!("scheduler {}: come back", arch::hart_id());
            let p = core::mem::replace(&mut c.process, None).unwrap();
            lst_pid = p.pid as usize + 1;
            if lst_pid >= NMAXPROCS {
                lst_pid = 0;
            }
//            info!("put back...");
            put_back_proc(p);
        } else {
            lst_pid = 0;
        }
    }
}
