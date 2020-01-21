// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(const_generics)]

use user::println;
use user::syscall::{fork, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let mut i : u64 = 0;
    let ptr = &mut i as *mut u64;
    // core::ptr::write_volatile(0x8000 as *mut u8, 0);
    for _ in 0..100 {
        core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) + 1);
    }
    for _ in 0..100 {
        println!("running init...");
    }
    fork();
    exit(0);
}
