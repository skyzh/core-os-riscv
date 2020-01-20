use core::ptr::null_mut;

#[repr(C)]
#[repr(align(4096))]
pub struct TrapFrame {
    pub regs: [usize; 32],   // 0 - 255
    pub fregs: [usize; 32],  // 256 - 511
    pub satp: usize,         // 512 - 519
    pub sp: usize,           // 520
    pub hartid: usize,       // 528
    pub trap: usize,         // 536 Address of usertrap
    pub epc: usize           // 544
}

impl TrapFrame {
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
