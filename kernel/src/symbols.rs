// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Linker-script constants and RISC-V constants
//! (This module may be viewed as memory layout definitions)

mod gen;
pub use gen::*;
use core::ops::Range;
use crate::println;

extern "C" {
	/// `uservec` function in `trampoline.S`
	pub fn uservec();
	/// `userret` function in `trampoline.S`
	pub fn userret();
	/// `kernelvec` function in `kernelvec.S`
	pub fn kernelvec();
	/// `m_trap_vector` function in `trap.S`
	pub fn timervec();
}

/// Page order
pub const PAGE_ORDER: usize = 12;

/// Page size
pub const PAGE_SIZE: usize = 1 << PAGE_ORDER;

/// Maximum virtual address supported on Sv39
pub const MAXVA: usize = 1 << (9 + 9 + 9 + 12 - 1);

/// Address to map kernel and user trampoline
pub const TRAMPOLINE_START: usize = MAXVA - PAGE_SIZE;

/// Address to map trapframe
pub const TRAPFRAME_START: usize = TRAMPOLINE_START - PAGE_SIZE;

/// Maximum supported CPU on machine
pub const NCPUS : usize = 8;

/// Maximum process on machine.
pub const NMAXPROCS: usize = 256;

/// Scheduler timer interrupt interval
pub const SCHEDULER_INTERVAL: usize = 1_000_000;

pub unsafe fn bss_range() -> Range<*mut usize> {
    Range {
        start: BSS_START as *mut usize,
        end: BSS_END as *mut usize,
    }
}

pub fn print_map_symbols() {
	println!("TEXT:   0x{:x} -> 0x{:x}", TEXT_START(), TEXT_END());
	println!("RODATA: 0x{:x} -> 0x{:x}", RODATA_START(), RODATA_END());
	println!("DATA:   0x{:x} -> 0x{:x}", DATA_START(), DATA_END());
	println!("BSS:    0x{:x} -> 0x{:x}", BSS_START(), BSS_END());
	println!(
		"STACK:  0x{:x} -> 0x{:x}",
		KERNEL_STACK_START(), KERNEL_STACK_END()
	);
	println!(
		"HEAP:   0x{:x} -> 0x{:x}",
		HEAP_START(),
		HEAP_START() + HEAP_SIZE()
	);
}
