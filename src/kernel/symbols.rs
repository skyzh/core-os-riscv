// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

pub use crate::symbols_gen::*;
use core::ops::Range;

extern "C" {
    pub fn uservec();
}

pub const PAGE_SIZE: usize = 1 << 12;
pub const MAXVA: usize = (1 << (9 + 9 + 9 + 12 - 1));
pub const TRAMPOLINE_START: usize = MAXVA - PAGE_SIZE;
pub const TRAPFRAME_START: usize = TRAMPOLINE_START - PAGE_SIZE;

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
