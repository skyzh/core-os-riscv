// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![allow(non_camel_case_types)]

use core::ptr::null_mut;

/// Trap frame contains information for switching to
/// and switching back from user space
#[repr(C)]
#[repr(align(4096))]
#[derive(Clone)]
pub struct TrapFrame {
    /// integer registers
    pub regs: [usize; 32],   // 0 - 255
    /// floating point registers
    pub fregs: [usize; 32],  // 256 - 511
    /// kernel satp register
    pub satp: usize,         // 512 - 519
    /// kernel sp
    pub sp: usize,           // 520
    /// kernel hartid
    pub hartid: usize,       // 528
    /// `usertrap` function address
    pub trap: usize,         // 536
    /// sret target address
    pub epc: usize           // 544
}

impl TrapFrame {
    /// create an initialized trapframe
    pub const fn zero() -> Self {
        TrapFrame {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            sp: 0,
            hartid: 0,
            trap: 0,
            epc: 0
        }
    }
}

/// Mapping between register name and trapframe `regs` array index
pub enum Register {
    zero = 0,
    ra = 1,
    sp = 2,
    gp = 3,
    tp = 4,
    t0 = 5,
    t1 = 6,
    t2 = 7,
    s0 = 8,
    s1 = 9,
    a0 = 10,
    a1 = 11,
    a2 = 12,
    a3 = 13,
    a4 = 14,
    a5 = 15,
    a6 = 16,
    a7 = 17
}
