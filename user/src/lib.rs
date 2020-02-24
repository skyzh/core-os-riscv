// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! User-space library

#![no_std]
#![feature(global_asm)]

pub mod print;
pub mod syscall;
pub mod constant;
mod syscall_internal;

use core::panic::PanicInfo;

#[cfg(target_pointer_width = "32")]
const HEX_WIDTH: usize = 10;
#[cfg(target_pointer_width = "64")]
const HEX_WIDTH: usize = 20;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
	loop {
	}
}
