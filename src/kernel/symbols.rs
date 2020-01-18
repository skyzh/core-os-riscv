// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use core::ops::Range;

extern "C" {
    pub static HEAP_START: usize;
    pub static HEAP_SIZE: usize;
    pub static TEXT_START: usize;
    pub static TEXT_END: usize;
    pub static RODATA_START: usize;
    pub static RODATA_END: usize;
    pub static DATA_START: usize;
    pub static DATA_END: usize;
    pub static BSS_START: usize;
    pub static BSS_END: usize;
    pub static KERNEL_STACK_START: usize;
    pub static KERNEL_STACK_END: usize;
    pub static TRAMPOLINE_TEXT_START: usize;
    pub fn uservec();
}

pub const PAGE_SIZE: usize = 1 << 12;
pub const MAXVA: usize =  (1 << (9 + 9 + 9 + 12 - 1));
pub const TRAMPOLINE_START : usize = MAXVA - PAGE_SIZE;
pub const TRAPFRAME_START:usize = TRAMPOLINE_START - PAGE_SIZE;

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
