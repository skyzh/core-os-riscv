// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],   // 0 - 255
    pub fregs: [usize; 32],  // 256 - 511
    pub satp: usize,         // 512 - 519
    pub sp: *mut u8,         // 520
    pub hartid: usize,       // 528
    pub trap: usize,         // 536
    pub epc: usize           // 544
}

use core::ptr::{null_mut, null};

impl TrapFrame {
    pub const fn zero() -> Self {
        TrapFrame {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            sp: null_mut(),
            hartid: 0,
            trap: 0,
            epc: 0
        }
    }
}

pub const NCPUS : usize = 8;

pub static mut KERNEL_TRAP_FRAME: [TrapFrame; NCPUS] = [TrapFrame::zero(); NCPUS];

pub const fn build_satp(mode: usize, asid: usize, addr: usize) -> usize {
    (mode as usize) << 60 | (asid & 0xffff) << 44 | (addr >> 12) & 0xff_ffff_ffff
}
