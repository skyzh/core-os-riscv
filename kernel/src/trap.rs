// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::{println, info};
use crate::process::{TrapFrame, self};
use crate::cpu;
use crate::symbols::*;
use crate::page;

#[no_mangle]
extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: &mut TrapFrame,
) -> usize {
    // We're going to handle all traps in machine mode. RISC-V lets
    // us delegate to supervisor mode, but switching out SATP (virtual memory)
    // gets hairy.
    let is_async = {
        if cause >> 63 & 1 == 1 {
            true
        } else {
            false
        }
    };
    // The cause contains the type of trap (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    if is_async {
        // Asynchronous trap
        match cause_num {
            3 => {
                // Machine software
                println!("Machine software interrupt CPU#{}", hart);
            }
            7 => unsafe {
                info!("Timer interrupt interrupt CPU#{}", hart);
                // Machine timer
                let mtimecmp = 0x0200_4000 as *mut u64;
                let mtime = 0x0200_bff8 as *const u64;
                // The frequency given by QEMU is 10_000_000 Hz, so this sets
                // the next interrupt to fire one second from now.
                mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
            },
            11 => {
                // Machine external (interrupt from Platform Interrupt Controller (PLIC))
                println!("Machine external interrupt CPU#{}", hart);
            }
            _ => {
                panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    } else {
        // Synchronous trap
        match cause_num {
            2 => {
                // Illegal instruction
                panic!(
                    "Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            8 => {
                // Environment (system) call from User mode
                println!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            }
            9 => {
                // Environment (system) call from Supervisor mode
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            }
            11 => {
                // Environment (system) call from Machine mode
                panic!("E-call from Machine mode! CPU#{} -> 0x{:08x}\n", hart, epc);
            }
            // Page faults
            12 => {
                // Instruction page fault
                println!(
                    "Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            13 => {
                // Load page fault
                println!(
                    "Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            15 => {
                // Store page fault
                println!(
                    "Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            _ => {
                panic!("Unhandled sync trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    };
    // Finally, return the updated program counter
    return_pc
}

#[no_mangle]
pub extern "C" fn usertrap() {
    // unsafe { crate::cpu::intr_on(); }
    info!("user trap.");
    crate::wait_forever();
}

#[inline]
pub fn trampoline_userret(tf: usize, satp_val: usize) -> ! {
    let uservec_offset = userret as usize - unsafe { TRAMPOLINE_TEXT_START };
    let fn_addr = (TRAMPOLINE_START + uservec_offset) as *const ();
    let fn_addr: extern "C" fn(usize, usize) -> usize = unsafe { core::mem::transmute(fn_addr) };
    (fn_addr)(tf, satp_val);
    crate::wait_forever()
}

pub fn usertrapret() -> ! {
    use riscv::register::*;

    let mut proc_cpu = process::CPUS[cpu::hart_id()].lock();
    cpu::intr_off();

    // send syscalls, interrupts, and exceptions to trampoline.S
    unsafe {
        stvec::write(
            (uservec as usize - TRAMPOLINE_TEXT_START) + TRAMPOLINE_START,
            stvec::TrapMode::Direct,
        );
    }

    // set up trapframe values that uservec will need when
    // the process next re-enters the kernel.
    let mut process = process::PROCS[proc_cpu.process_id as usize].lock();
    process.trapframe.satp = proc_cpu.kernel_trapframe.satp;
    process.trapframe.sp = proc_cpu.kernel_trapframe.sp;
    process.trapframe.trap = crate::trap::usertrap as usize;
    process.trapframe.hartid = proc_cpu.kernel_trapframe.hartid;

    // println!("trap 0x{:x}", proc_cpu.process.trapframe.trap);
    
    // set S Previous Privilege mode to User.
    unsafe {
        sstatus::set_spie();
        sstatus::set_spp(sstatus::SPP::User);
    }

    // set S Exception Program Counter to the saved user pc.
    sepc::write(process.trapframe.epc);

    // tell trampoline.S the user page table to switch to.
    let satp_val;
    {
        let root_ppn = &mut process.pgtable as *mut page::Table as usize;
        satp_val = crate::cpu::build_satp(8, 0, root_ppn);
    }

    // jump to trampoline.S at the top of memory, which 
    // switches to the user page table, restores user registers,
    // and switches to user mode with sret.

    println!("jumping to trampoline 0x{:x} 0x{:x}...", &process.trapframe as *const _ as usize, TRAPFRAME_START);
    trampoline_userret(TRAPFRAME_START, satp_val)
}
