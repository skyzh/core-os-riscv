// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![no_std]
#![feature(asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(const_generics)]

use core::{fmt, panic::PanicInfo};

mod user;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let mut i : u64 = 0;
    let ptr = &mut i as *mut u64;
    // core::ptr::write_volatile(0x8000 as *mut u8, 0);
    for i in 0..100 {
        core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) + 1);
    }
    user::__write(0, "233333", 6);
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
	loop {
	}
}