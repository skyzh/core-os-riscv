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

use user::{println, print, format};
use user::syscall::{fork, exit, exec, write};
use core::ptr::null;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    loop { println!("test1!"); }
    exit(0);
}
