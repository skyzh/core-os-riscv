// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V hart boot and initialize

use riscv::{asm, register::*};
use crate::process::my_cpu;
use crate::page::Page;
use crate::symbols::{NCPUS, print_map_symbols};
use crate::clint::CLINT_MTIMECMP;
use crate::arch::{intr_get, hart_id};
use crate::{clint, plic, mem, uart, process, spinlock, trap};
use crate::info;

/// Initialize kernel page table and drivers in machine mode,
/// and prepare to switch to supervisor mode
#[no_mangle]
unsafe extern "C" fn kinit() {
    // next mode is supervisor mode
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    // mret jump to kmain
    mepc::write(kmain as usize);
    // disable paging
    asm!("csrw satp, zero");
    // delegate all interrupts and exceptions to supervisor mode
    asm!("li t0, 0xffff");
    asm!("csrw medeleg, t0");
    asm!("li t0, 0xffff");
    asm!("csrw mideleg, t0");
    // save cpuid to tp
    asm!("csrr a1, mhartid");
    asm!("mv tp, a1");
    // set up timer interrupt
    clint::timer_init();
    // switch to supervisor mode
    asm!("mret");
}

use spinlock::Mutex;

/// Controls whether other harts may start boot procedure
static mut may_boot: bool = false;

/// Initialize hart, start first process and begin scheduling
#[no_mangle]
extern "C" fn kmain() -> ! {
    if hart_id() == 0 {
        mem::alloc_init();
        uart::UART().lock().init();
        info!("booting core-os on hart {}...", hart_id());
        info!("drivers:");
        info!("  UART... \x1b[0;32minitialized\x1b[0m");
        plic::PLIC().lock().init(plic::UART0_IRQ);
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        unsafe { mem::init(); }
        mem::hartinit();
        info!("page table configured");
        unsafe { trap::init(); }
        unsafe { process::init(); }
        info!("  Trap... \x1b[0;32minitialized\x1b[0m");
        info!("  Timer... \x1b[0;32minitialized\x1b[0m");
        plic::init();
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        info!("  Interrupt... \x1b[0;32minitialized\x1b[0m");
        process::init_proc();
        unsafe {
            asm!("fence");
            may_boot = true
        }
    } else {
        loop {
            if unsafe { may_boot } == true {
                break;
            }
        }
        info!("hart {} booting", hart_id());
        mem::hartinit();
        unsafe { trap::init(); }
        plic::init();
    }
    process::scheduler()
}
