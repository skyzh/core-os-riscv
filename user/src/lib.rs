// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![no_std]
#![feature(global_asm)]

pub mod print;
pub mod syscall;
mod syscall_internal;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
	loop {
	}
}
