pub const CLINT_BASE: usize = 0x200_0000;
pub const CLINT_MTIMECMP_BASE: usize = CLINT_BASE + 0x4000;
pub const fn CLINT_MTIMECMP(hart: usize) -> usize { CLINT_MTIMECMP_BASE + 8 * hart }
pub const CLINT_MTIME_BASE: usize = CLINT_BASE + 0xBFF8;
