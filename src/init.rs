
use crate::memory::zero_volatile;
use crate::link::bss_range;
use core::ops::Range;

pub unsafe fn runtime_init() {
    zero_volatile(bss_range());
    kernel_init();
}

pub fn kernel_init() {
    //  ELR_EL2.set(crate::runtime_init::runtime_init as *const () as u64);
	loop {}
}
