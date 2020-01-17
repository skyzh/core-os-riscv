#![no_main]
#![no_std]

use core::{fmt, panic::PanicInfo};

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let mut i : u64 = 0;
    let ptr = &mut i as *mut u64;
    loop {
        core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) + 1);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
