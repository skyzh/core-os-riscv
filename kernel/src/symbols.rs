// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub use crate::symbols_gen::*;
use core::ops::Range;

extern "C" {
	pub fn uservec();
	pub fn userret();
}

pub const PAGE_ORDER: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_ORDER;
pub const MAXVA: usize = (1 << (9 + 9 + 9 + 12 - 1));
pub const TRAMPOLINE_START: usize = MAXVA - PAGE_SIZE;
pub const TRAPFRAME_START: usize = TRAMPOLINE_START - PAGE_SIZE;
pub const NCPUS : usize = 8;
pub const NMAXPROCS: usize = 256;

pub unsafe fn bss_range() -> Range<*mut usize> {
    extern "C" {
        // Boundaries of the .bss section, provided by linker script symbols.
        static mut __bss_start: usize;
        static mut __bss_end: usize;
    }

    Range {
        start: &mut __bss_start,
        end: &mut __bss_end,
    }
}

use crate::println;

pub fn print_map_symbols() {
    unsafe {
		println!("TEXT:   0x{:x} -> 0x{:x}", TEXT_START, TEXT_END);
		println!("RODATA: 0x{:x} -> 0x{:x}", RODATA_START, RODATA_END);
		println!("DATA:   0x{:x} -> 0x{:x}", DATA_START, DATA_END);
		println!("BSS:    0x{:x} -> 0x{:x}", BSS_START, BSS_END);
		println!(
			"STACK:  0x{:x} -> 0x{:x}",
			KERNEL_STACK_START, KERNEL_STACK_END
		);
		println!(
			"HEAP:   0x{:x} -> 0x{:x}",
			HEAP_START,
			HEAP_START + HEAP_SIZE
		);
	}
}