// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use riscv::register::*;

pub const fn build_satp(mode: usize, asid: usize, addr: usize) -> usize {
    (mode as usize) << 60 | (asid & 0xffff) << 44 | (addr >> 12) & 0xff_ffff_ffff
}

pub unsafe fn intr_on() {
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();
    sstatus::set_sie();
}

pub unsafe fn intr_off() {
    sstatus::clear_sie();
}

#[inline(always)]
pub fn hart_id() -> usize {
    let hart_id: usize = 0;
    unsafe { asm!("mv $0, tp" :: "r"(hart_id)); }
    hart_id
}
