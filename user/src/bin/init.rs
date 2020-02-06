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
use user::syscall::{fork, exec};

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    println!("ready to fork!");
    let p = fork();
    if p == 0 {
        println!("subprocess calling test1");
        exec("/test1", &["test1", "test2"]);
    } else {
        loop {}
    }
}
