// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V hart boot and initialize

use riscv::{asm, register::*};
use crate::arch::{hart_id, wait_forever};
use crate::{clint, plic, mem, uart, process, spinlock, trap, virtio};
use crate::info;
use crate::jump::*;

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

/// Controls whether other harts may start boot procedure
static mut MAY_BOOT: bool = false;

/// Initialize hart, start first process and begin scheduling
#[no_mangle]
extern "C" fn kmain() -> ! {
    if hart_id() == 0 {
        unsafe { uart::init(); }
        info!("booting core-os on hart {}...", hart_id());
        info!("  UART... \x1b[0;32minitialized\x1b[0m");
        unsafe { mem::init(); }
        info!("  kernel page table... \x1b[0;32minitialized\x1b[0m");
        unsafe { virtio::init(); }
        info!("  virt-io... \x1b[0;32minitialized\x1b[0m");
        unsafe { plic::init(); }
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        mem::hartinit();
        info!("kernel page table configured");
        info!("  Trap... \x1b[0;32minitialized\x1b[0m");
        info!("  Timer... \x1b[0;32minitialized\x1b[0m");
        plic::hartinit();
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        unsafe { trap::hartinit(); }
        info!("  Interrupt... \x1b[0;32minitialized\x1b[0m");
        unsafe { process::init(); }
        process::init_proc();
        unsafe {
            asm!("fence");
            MAY_BOOT = true
        }
    } else {
        loop {
            if unsafe { MAY_BOOT } == true {
                break;
            }
        }
        info!("hart {} booting", hart_id());
        mem::hartinit();
        unsafe { trap::hartinit(); }
        plic::hartinit();
    }
    process::scheduler()
}
