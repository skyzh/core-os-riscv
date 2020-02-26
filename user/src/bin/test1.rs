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
use user::syscall::{exit, fork, exec, open, read, write};
use user::constant::STDOUT;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let p = fork();
    if p == 0 {
        println!("forking test2...");
        exec("/test2", &["test1", "test2"]);
    }
    println!("test1 running...");
    let fd = open("/test.txt", 0);
    let mut data = [0; 32];
    read(fd, &mut data);
    write(STDOUT, &data);
    exit(0);
}
