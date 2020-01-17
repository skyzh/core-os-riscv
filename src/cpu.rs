// trap.rs
// Trap routines
// Stephen Marz
// 10 October 2019

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],   // 0 - 255
    pub fregs: [usize; 32],  // 256 - 511
    pub satp: usize,         // 512 - 519
    pub trap_stack: *mut u8, // 520
    pub hartid: usize,       // 528
}

use core::ptr::null_mut;

impl TrapFrame {
    pub const fn zero() -> Self {
        TrapFrame {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            trap_stack: null_mut(),
            hartid: 0,
        }
    }
}

pub static mut KERNEL_TRAP_FRAME: [TrapFrame; 8] = [TrapFrame::zero(); 8];

pub const fn build_satp(mode: usize, asid: usize, addr: usize) -> usize {
    (mode as usize) << 60 | (asid & 0xffff) << 44 | (addr >> 12) & 0xff_ffff_ffff
}
