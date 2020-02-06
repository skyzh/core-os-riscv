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
use user::syscall::{exit, fork, exec};

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let p = fork();
    if p == 0 {
        println!("forking test3...");
        exec("/test3", &["test1", "test2"]);
    }
    println!("test2 running...");
    exit(0);
}
