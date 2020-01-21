// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::wait_forever;
use crate::arch;
use crate::trap::usertrapret;
use crate::symbols::*;
use crate::process::{PROCS, ProcessState, swtch, Register, Context, my_cpu};
use crate::info;

pub fn scheduler() -> ! {
    loop {
        arch::intr_on();
        let c = my_cpu();
        for pid in 0..NMAXPROCS {
            let mut p = PROCS[pid].lock();
            if p.state == ProcessState::RUNNABLE {
                info!("scheduler: switching to {}", pid);
                p.state = ProcessState::RUNNING;
                let ctx = core::mem::replace(&mut p.context, Context::zero());
                drop(p);
                c.process_id = pid as i64;
                info!("swtch to proc");
                swtch(&mut c.scheduler_context, ctx);
                info!("come back");
                c.process_id = -1;
            }
        }
    }
}
