// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![no_std]
#![feature(panic_info_message, asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(alloc_error_handler)]
#![feature(box_syntax)]
#![feature(alloc_prelude)]
#![allow(dead_code)]

#[macro_use]
extern crate alloc;

// This is experimental and requires alloc_prelude as a feature
use alloc::prelude::v1::*;

mod arch;
mod elf;
mod fs;
mod mem;
mod spinlock;
mod page;
mod print;
mod process;
mod symbols;
mod trap;
mod uart;
mod plic;
mod clint;
mod syscall;

use riscv::{asm, register::*};
use crate::process::my_cpu;
use crate::page::Page;
use crate::symbols::NCPUS;
use crate::clint::CLINT_MTIMECMP;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::arch::intr_get;

#[no_mangle]
extern "C" fn eh_personality() {}

/// Panic handler
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_println!("Aborting: ");
    if let Some(p) = info.location() {
        panic_println!(
			"line {}, file {}: {}",
			p.line(),
			p.file(),
			info.message().unwrap()
		);
    } else {
        panic_println!("no information available.");
    }
    abort();
}

/// Abort function
#[no_mangle]
extern "C" fn abort() -> ! {
    arch::wait_forever();
}

/// Initialize kernel page table and drivers in machine mode,
/// and prepare to switch to supervisor mode
#[no_mangle]
unsafe extern "C" fn kinit() {
    use riscv::register::*;
    // next mode is supervisor mode
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    // mret jump to kmain
    mepc::write(kmain as usize);
    // disable paging
    asm!("csrw satp, zero");
    // delegate all interrupts and exceptions to supervisor mode
    asm!("li t0, 0xffff");
    asm!("csrw medeleg, t0");
    asm!("li t0, 0xff0f");
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

/// Controls whether other harts can start boot procedure
static may_boot: Mutex<bool> = Mutex::new(false);

/// Initialize hart, start first process and begin scheduling
#[no_mangle]
extern "C" fn kmain() -> ! {
    use arch::hart_id;
    if hart_id() == 0 {
        mem::alloc_init();
        uart::UART().lock().init();
        info!("booting core-os on hart {}...", hart_id());
        info!("drivers:");
        info!("  UART... \x1b[0;32minitialized\x1b[0m");
        plic::PLIC().lock().init(plic::UART0_IRQ);
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        mem::init();
        mem::hartinit();
        info!("page table configured");
        unsafe { trap::init(); }
        info!("  Trap... \x1b[0;32minitialized\x1b[0m");
        info!("  Timer... \x1b[0;32minitialized\x1b[0m");
        plic::init();
        info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
        info!("  Interrupt... \x1b[0;32minitialized\x1b[0m");
        process::init_proc();
        *may_boot.lock() = true;
    } else {
        loop {
            let l = may_boot.lock();
            if *l == true {
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
