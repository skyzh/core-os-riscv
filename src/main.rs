#![no_std]
#![no_main]
#![feature(panic_info_message,asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]

global_asm!(include_str!("asm/trap.S"));
global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/ld_symbols.S"));

mod init;
mod link;
mod memory;
mod alloc;
mod nulllock;
mod print;
mod uart;

use riscv::{register::*, asm};

#[no_mangle]
pub unsafe extern "C" fn start() -> ! { 
	memory::zero_volatile(link::bss_range());
	/*
	mstatus::set_fs(mstatus::FS::Dirty);
	mstatus::set_mpp(mstatus::MPP::Supervisor);
	mstatus::set_spp(mstatus::SPP::Supervisor);
	mepc::write(kmain as *const () as usize);
	extern "C" {
        static asm_trap_vector: usize;
    }
	mtvec::write(asm_trap_vector as *const () as usize, mtvec::TrapMode::Vectored);
	*/
	asm!("li t0, (0b11 << 11) | (1 << 7) | (1 << 3)");
	asm!("csrw	mstatus, t0");
	asm!("la t1, kmain");
	asm!("csrw	mepc, t1");
	asm!("la t2, asm_trap_vector");
	asm!("csrw	mtvec, t2");
	asm!("li t3, (1 << 3) | (1 << 7) | (1 << 11)");
	asm!("csrw	mie, t3");
	asm!("mret");
	// TODO: specify return addr
	loop {
		asm::wfi();
	}
}

#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	print!("Aborting: ");
	if let Some(p) = info.location() {
		println!(
				"line {}, file {}: {}",
				p.line(),
				p.file(),
				info.message().unwrap()
				);
	}
	else {
		println!("no information available.");
	}
	abort();
}
#[no_mangle]
extern "C"
fn abort() -> ! {
	loop {
		unsafe {
			asm!("wfi"::::"volatile");
		}
	}
}

// ///////////////////////////////////
// / CONSTANTS
// ///////////////////////////////////

// ///////////////////////////////////
// / ENTRY POINT
// ///////////////////////////////////
#[no_mangle]
extern "C"
fn kmain() {
	// Main should initialize all sub-systems and get
	// ready to start scheduling. The last thing this
	// should do is start the timer.

	// Let's try using our newly minted UART by initializing it first.
	// The UART is sitting at MMIO address 0x1000_0000, so for testing
	// now, lets connect to it and see if we can initialize it and write
	// to it.
	uart::UART.lock().init();

	// Now test println! macro!
	println!("mhartid {}", mhartid::read());

	let ptr = alloc::ALLOCATOR.lock().allocate(64 * 4096);
	let ptr = alloc::ALLOCATOR.lock().allocate(1);
	let ptr2 = alloc::ALLOCATOR.lock().allocate(1);
	let ptr = alloc::ALLOCATOR.lock().allocate(1);
	alloc::ALLOCATOR.lock().deallocate(ptr);
	let ptr = alloc::ALLOCATOR.lock().allocate(1);
	let ptr = alloc::ALLOCATOR.lock().allocate(1);
	let ptr = alloc::ALLOCATOR.lock().allocate(1);
	alloc::ALLOCATOR.lock().deallocate(ptr2);
	alloc::ALLOCATOR.lock().debug();
}
