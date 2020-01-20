#![no_std]
#![feature(global_asm)]

use core::panic::PanicInfo;

global_asm!(include_str!("usys.S"));

extern "C" {
    pub fn __write(fd: i32, content: *const u8, size: i32) -> i32;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
	loop {
	}
}
