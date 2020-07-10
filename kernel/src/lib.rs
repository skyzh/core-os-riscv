// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//！Kernel code

#![no_std]
#![feature(panic_info_message, asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(alloc_error_handler)]
#![feature(box_syntax)]
#![feature(alloc_prelude)]
#![feature(llvm_asm)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

// This is experimental and requires alloc_prelude as a feature
use alloc::prelude::v1::*;

mod arch;
mod elf;
mod mem;
mod spinlock;
mod page;
mod print;
mod process;
mod symbols;
mod trap;
mod uart;
mod plic;
mod clint;
mod syscall;
mod start;
mod jump;
mod virtio;
mod intr;
mod test;
mod sleeplock;
mod file;

#[no_mangle]
extern "C" fn eh_personality() {}

/// Panic handler
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_println!("hart {} aborting: ", arch::hart_id());
    if let Some(p) = info.location() {
        panic_println!(
			"line {}, file {}: {}",
			p.line(),
			p.file(),
			info.message().unwrap()
		);
    } else {
        panic_println!("no information available.");
    }
    abort();
}

/// Abort function
#[no_mangle]
extern "C" fn abort() -> ! {
    arch::wait_forever();
}
