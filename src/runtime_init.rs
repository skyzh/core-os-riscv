
use crate::memory::zero_volatile;
use crate::link::bss_range;
use core::ops::Range;

pub unsafe fn runtime_init() {
    zero_volatile(bss_range());
    crate::kernel_init();
}
