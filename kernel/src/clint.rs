// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V Core Local Interrupter

#![allow(non_snake_case)]

use crate::symbols::{NCPUS, SCHEDULER_INTERVAL};
use crate::println;
use crate::arch::{hart_id, sp};

pub const CLINT_BASE: usize = 0x200_0000;
pub const CLINT_MTIMECMP_BASE: usize = CLINT_BASE + 0x4000;
pub const fn CLINT_MTIMECMP(hart: usize) -> usize { CLINT_MTIMECMP_BASE + 8 * hart }
pub const CLINT_MTIME_BASE: usize = CLINT_BASE + 0xBFF8;

/// space for timer trap to save information.
static mut MSCRATCH0: [[u64; 8]; NCPUS] = [[0; 8]; NCPUS];

/// Initialize machine-mode timer interrupt
pub unsafe fn timer_init() {
    use riscv::register::*;
    let id = mhartid::read();
    let interval = SCHEDULER_INTERVAL as u64;
    let mtimecmp = CLINT_MTIMECMP(id) as *mut u64;
    let mtime = CLINT_MTIME_BASE as *const u64;
    mtimecmp.write_volatile(mtime.read_volatile() + interval);
    let scratch = &mut MSCRATCH0[id];

    // space for timer trap to save information.
    mscratch::write(scratch.as_mut_ptr() as usize);
    scratch[3] = mtime as u64;
    scratch[4] = mtimecmp as u64;
    scratch[5] = interval;

    // set machine-mode trap handler as timervec in kernelvec.S
    mtvec::write(crate::symbols::timervec as usize, mtvec::TrapMode::Direct);

    // enable machine-mode interrupts.
    mstatus::set_mie();

    // enable machine-mode timer interrupt.
    mie::set_mtimer();
}

pub fn debug() {
    unsafe {
        println!("0x{:x} 0x{:x}, {}", MSCRATCH0.as_mut_ptr() as usize, MSCRATCH0[hart_id()][4], MSCRATCH0[hart_id()][5]);
    }
}
