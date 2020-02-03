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
#![feature(new_uninit)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_ref)]
#![allow(dead_code)]

#[macro_use]
extern crate alloc;

// This is experimental and requires alloc_prelude as a feature
use alloc::prelude::v1::*;

mod arch;
mod elf;
mod fs;
mod mem;
mod nulllock;
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
    if mhartid::read() != 0 {
        arch::wait_forever();
    }
    use riscv::register::*;
    // next mode is supervisor mode
    mstatus::set_sie();
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    // mret jump to kmain
    mepc::write(kmain as usize);
    // disable paging
    asm!("csrw satp, zero");
    // delegate all interrupts and exceptions to supervisor mode
    asm!("li t0, 0xffff");
    asm!("csrw medeleg, t0");
    asm!("csrw mideleg, t0");
    // save cpuid to tp
    asm!("csrr a1, mhartid");
    asm!("mv tp, a1");
    // setup machine mode interrupt
    mscratch::write(&mut (my_cpu().kernel_trapframe) as *mut _ as usize);
    mtvec::write(symbols::m_trap_vector as usize, mtvec::TrapMode::Direct);
    // switch to supervisor mode
    asm!("mret");
}

/// Initialize hart other than `0`
#[no_mangle]
extern "C" fn kinit_hart(hartid: usize) {
    arch::wait_forever();
    /*
    use process::TrapFrame;
    let mut cpu = my_cpu();
    let kernel_trapframe = &mut cpu.kernel_trapframe;
    mscratch::write(kernel_trapframe as *mut TrapFrame as usize);
    // We can't do the following until zalloc() is locked, but we
    // don't have locks, yet :( cpu::KERNEL_TRAP_FRAME[hartid].satp
    // = cpu::KERNEL_TRAP_FRAME[0].satp;
    // cpu::KERNEL_TRAP_FRAME[hartid].trap_stack = page::zalloc(1);
    kernel_trapframe.hartid = hartid;
    info!("{} initialized", hartid);
    arch::wait_forever();
    */
}

/// Start first process and begin scheduling
#[no_mangle]
extern "C" fn kmain() -> ! {
    if arch::hart_id() != 0 {
        arch::wait_forever();
    }
    // TODO: zero volatile bss
    // unsafe { mem::zero_volatile(symbols::bss_range()); }
    mem::alloc_init();
    uart::UART().lock().init();
    info!("Booting core-os...");
    info!("Drivers:");
    info!("  UART... \x1b[0;32minitialized\x1b[0m");
    plic::PLIC().lock().init(plic::UART0_IRQ);
    info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
    info!("Booting on hart {}", arch::hart_id());
    mem::init();
    info!("Initializing...");
    unsafe { trap::init(); }
    info!("  Trap... \x1b[0;32minitialized\x1b[0m");

    arch::intr_on();
    /*
    unsafe {
        let mtimecmp = clint::CLINT_MTIMECMP_BASE as *mut u64;
        let mtime = clint::CLINT_MTIME_BASE as *const u64;
        // The frequency given by QEMU is 10_000_000 Hz, so this sets
        // the next interrupt to fire one second from now.
        mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
    }
    */

    info!("  Timer... \x1b[0;32minitialized\x1b[0m");
    {
        let mut PLIC = plic::PLIC().lock();
        PLIC.enable(plic::UART0_IRQ);
        PLIC.set_threshold(0);
    }
    // PLIC.set_priority(plic::UART0_IRQ, 1);
    info!("  PLIC... \x1b[0;32minitialized\x1b[0m");
    /*
    loop {
        let x = uart::UART().lock().get();
        if let Some(_x) = x {
            print!("{}", _x as char);
        }
    }
    */
    // unsafe { core::ptr::write_volatile(0 as *mut u8, 0); }
    arch::wait_forever();
    process::init_proc();
    process::scheduler()
}
