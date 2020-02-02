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
extern "C" fn kinit() {
    // unsafe { mem::zero_volatile(symbols::bss_range()); }
    mem::init();
    uart::UART().lock().init();
    info!("Booting core-os...");
    info!("Drivers:");
    info!("  UART intialized");
    info!("Booting on hart {}", mhartid::read());
    use symbols::*;
    print_map_symbols();
    use page::EntryAttributes;
    use page::{Table, KERNEL_PGTABLE};
    let mut pgtable = KERNEL_PGTABLE().lock();
    pgtable.id_map_range(
        TEXT_START(),
        TEXT_END(),
        EntryAttributes::RX as usize,
    );
    pgtable.id_map_range(
        RODATA_START(),
        RODATA_END(),
        EntryAttributes::RX as usize,
    );
    pgtable.id_map_range(
        DATA_START(),
        DATA_END(),
        EntryAttributes::RW as usize,
    );
    pgtable.id_map_range(
        BSS_START(),
        BSS_END(),
        EntryAttributes::RW as usize,
    );
    pgtable.id_map_range(
        KERNEL_STACK_START(),
        KERNEL_STACK_END(),
        EntryAttributes::RW as usize,
    );
    pgtable.kernel_map(
        UART_BASE_ADDR,
        UART_BASE_ADDR,
        EntryAttributes::RW as usize,
    );
    pgtable.kernel_map(
        TRAMPOLINE_START,
        TRAMPOLINE_TEXT_START(),
        page::EntryAttributes::RX as usize,
    );
    pgtable.id_map_range(
        HEAP_START(),
        HEAP_START() + HEAP_SIZE(),
        EntryAttributes::RW as usize,
    );
    // CLINT
    //  -> MSIP
    pgtable.id_map_range(0x0200_0000, 0x0200_ffff, EntryAttributes::RW as usize);
    // PLIC
    pgtable.id_map_range(0x0c00_0000, 0x0c00_2000, EntryAttributes::RW as usize);
    pgtable.id_map_range(0x0c20_0000, 0x0c20_8000, EntryAttributes::RW as usize);
    use uart::*;
    /* TODO: use Rust primitives */
    use process::TrapFrame;
    let cpu = my_cpu();
    let kernel_trapframe = &mut cpu.kernel_trapframe;

    let root_ppn = &mut *pgtable as *mut Table as usize;
    let satp_val = arch::build_satp(8, 0, root_ppn);
    unsafe {
        mscratch::write(kernel_trapframe as *mut TrapFrame as usize);
    }
    kernel_trapframe.satp = satp_val;
    let stack_addr = mem::ALLOC().lock().allocate(PAGE_SIZE * 1024);
    kernel_trapframe.sp = stack_addr as usize + PAGE_SIZE * 1024;
    kernel_trapframe.hartid = 0;
    pgtable.id_map_range(
        stack_addr as usize,
        stack_addr as usize + mem::PAGE_SIZE,
        EntryAttributes::RW as usize,
    );
    unsafe {
        asm!("csrw satp, $0" :: "r"(satp_val));
        asm!("sfence.vma zero, zero");
    }
    info!("Page table set up, switching to supervisor mode");
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
    info!("Now in supervisor mode");
    /*
    unsafe {
        let mtimecmp = 0x0200_4000 as *mut u64;
        let mtime = 0x0200_bff8 as *const u64;
        // The frequency given by QEMU is 10_000_000 Hz, so this sets
        // the next interrupt to fire one second from now.
        mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
    }*/
    process::init_proc();
    process::scheduler()
}
