// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::{println, info, panic};
use crate::process::{TrapFrame, self, Process, CPU, my_proc, my_cpu, yield_cpu};
use crate::arch;
use crate::symbols::*;
use crate::page;
use crate::nulllock::Mutex;
use crate::syscall;

#[no_mangle]
extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    _status: usize,
    _frame: &mut TrapFrame,
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
    // info!("user trap");
    use riscv::register::*;
    if sstatus::read().spp() != sstatus::SPP::User {
        panic!("not from user mode");
    }
    // stvec::write()

    let p = my_proc();
    p.trapframe.epc = sepc::read();

    if scause::read().bits() == 8 {
        p.trapframe.epc += 4;
        arch::intr_on();
        syscall::syscall();
        yield_cpu();
    } else {
        panic!("unexpected scause");
    }

    usertrapret();
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
    let satp_val: usize;
    {
        use riscv::register::*;
        arch::intr_off();

        // send syscalls, interrupts, and exceptions to trampoline.S
        unsafe {
            stvec::write(
                (uservec as usize - TRAMPOLINE_TEXT_START) + TRAMPOLINE_START,
                stvec::TrapMode::Direct,
            );
        }

        // set up trapframe values that uservec will need when
        // the process next re-enters the kernel.
        let mut p = my_proc();
        let c = my_cpu();
        p.trapframe.satp = c.kernel_trapframe.satp;
        p.trapframe.sp = c.kernel_trapframe.sp;
        p.trapframe.trap = crate::trap::usertrap as usize;
        p.trapframe.hartid = c.kernel_trapframe.hartid;

        // println!("trap 0x{:x}", proc_cpu.process.trapframe.trap);

        // set S Previous Privilege mode to User.
        unsafe {
            sstatus::set_spie();
            sstatus::set_spp(sstatus::SPP::User);
        }

        // set S Exception Program Counter to the saved user pc.
        sepc::write(p.trapframe.epc);

        // tell trampoline.S the user page table to switch to.
        let root_ppn = &mut p.pgtable as *mut page::Table as usize;
        satp_val = crate::arch::build_satp(8, 0, root_ppn);
    }

    // jump to trampoline.S at the top of memory, which 
    // switches to the user page table, restores user registers,
    // and switches to user mode with sret.
    // println!("jumping to trampoline 0x{:x} 0x{:x}...", trap_frame_addr , TRAPFRAME_START);
    trampoline_userret(TRAPFRAME_START, satp_val)
}
