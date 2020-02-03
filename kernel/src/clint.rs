// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V Core Local Interrupter

use crate::symbols::NCPUS;

pub const CLINT_BASE: usize = 0x200_0000;
pub const CLINT_MTIMECMP_BASE: usize = CLINT_BASE + 0x4000;
pub const fn CLINT_MTIMECMP(hart: usize) -> usize { CLINT_MTIMECMP_BASE + 8 * hart }
pub const CLINT_MTIME_BASE: usize = CLINT_BASE + 0xBFF8;

/// space for timer trap to save information.
static mut mscratch0: [usize; NCPUS * 32] = [0; NCPUS * 32];

/// Initialize machine-mode timer interrupt
pub unsafe fn timer_init() {
    use riscv::register::*;
    let id = mhartid::read();
    let interval = 1_000_000;
    let mtimecmp = CLINT_MTIMECMP_BASE as *mut u64;
    let mtime = CLINT_MTIME_BASE as *const u64;
    mtimecmp.write_volatile(mtime.read_volatile() + interval);

    // space for timer trap to save information.
    let base_addr = &mut mscratch0 as *mut _ as usize + 32 * id;
    mscratch::write(base_addr);
    let scratch = base_addr as *mut usize;
    scratch.add(4).write_volatile(CLINT_MTIMECMP(id));
    scratch.add(5).write_volatile(interval as usize);

    // set machine-mode trap handler as timervec in kernelvec.S
    mtvec::write(crate::symbols::timervec as usize, mtvec::TrapMode::Direct);

    // enable machine-mode interrupts.
    mstatus::set_mie();

    // enable machine-mode timer interrupt.
    mie::set_mtimer();
}
