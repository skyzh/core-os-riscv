use crate::wait_forever;
use crate::cpu;
use crate::trap::{usertrapret, my_cpu};
use crate::symbols::*;
use crate::process::{PROCS, ProcessState, swtch, Register, Context};
use crate::info;

pub fn scheduler() -> ! {
    loop {
        cpu::intr_on();
        for pid in 0..NMAXPROCS {
            let mut p = PROCS[pid].lock();
            if p.state == ProcessState::RUNNABLE {
                info!("scheduler: switching to {}", pid);
                p.state = ProcessState::RUNNING;
                let context = core::mem::replace(&mut p.context, Context::zero());
                drop(p);
                my_cpu().lock().process_id = pid as i64;
                my_cpu().lock().scheduler_context = swtch(context);
                my_cpu().lock().process_id = -1;
            }
        }
    }
}
