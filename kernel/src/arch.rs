// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V related functions

use core::time::Duration;
use crate::panic;
use riscv::register::*;
use crate::symbols::*;

/// Get current time from MMIO
pub fn time() -> Duration {
    let mtime = 0x0200_bff8 as *const u64;
    Duration::from_nanos(unsafe { mtime.read_volatile() } * 100)
}

/// Build satp value from mode, asid and page table base addr
pub fn build_satp(mode: usize, asid: usize, addr: usize) -> usize {
    if addr % PAGE_SIZE != 0 {
        panic!("satp not aligned!");
    }
    (mode as usize) << 60 | (asid & 0xffff) << 44 | (addr >> 12) & 0xff_ffff_ffff
}

/// Enable interrupt
pub fn intr_on() {
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
        sstatus::set_sie();
    }
}

/// Turn off interrupt
pub fn intr_off() {
    unsafe {
        sstatus::clear_sie();
    }
}

#[inline(always)]
pub fn hart_id() -> usize {
    let mut hart_id: usize = 0;
    unsafe { asm!("mv $0, tp" :: "r"(hart_id)); }
    hart_id
}

extern "C" { fn __sp() -> usize; }

pub fn sp() -> usize {
    unsafe { __sp() }
}

pub fn wait_forever() -> ! {
    loop {
        unsafe {
            riscv::asm::wfi();
        }
    }
}
