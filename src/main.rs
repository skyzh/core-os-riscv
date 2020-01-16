#![no_std]
#![no_main]
#![feature(panic_info_message, asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]

global_asm!(include_str!("asm/trap.S"));
global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/ld_symbols.S"));

mod alloc;
mod constant;
mod init;
mod memory;
mod mmu;
mod nulllock;
mod print;
mod uart;

use riscv::{asm, register::*};

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
	} else {
		println!("no information available.");
	}
	abort();
}
#[no_mangle]
extern "C" fn abort() -> ! {
	loop {
		unsafe {
			asm!("wfi"::::"volatile");
		}
	}
}
#[no_mangle]
extern "C" fn kinit() -> usize {
	// memory::zero_volatile(constant::bss_range());
	uart::UART.lock().init();

	println!("mhartid {}", mhartid::read());

	use constant::*;
	unsafe {
		println!("TEXT:   0x{:x} -> 0x{:x}", TEXT_START, TEXT_END);
		println!("RODATA: 0x{:x} -> 0x{:x}", RODATA_START, RODATA_END);
		println!("DATA:   0x{:x} -> 0x{:x}", DATA_START, DATA_END);
		println!("BSS:    0x{:x} -> 0x{:x}", BSS_START, BSS_END);
		println!(
			"STACK:  0x{:x} -> 0x{:x}",
			KERNEL_STACK_START, KERNEL_STACK_END
		);
		println!(
			"HEAP:   0x{:x} -> 0x{:x}",
			HEAP_START,
			HEAP_START + HEAP_SIZE
		);
	}
	use mmu::EntryAttributes;
	use mmu::{Table, KERNEL_PGTABLE};
	let mut pgtable = KERNEL_PGTABLE.lock();
	pgtable.id_map_range(
		unsafe { TEXT_START },
		unsafe { TEXT_END },
		EntryAttributes::RX as usize,
	);
	pgtable.id_map_range(
		unsafe { RODATA_START },
		unsafe { RODATA_END },
		EntryAttributes::RX as usize,
	);
	pgtable.id_map_range(
		unsafe { DATA_START },
		unsafe { DATA_END },
		EntryAttributes::RW as usize,
	);
	pgtable.id_map_range(
		unsafe { BSS_START },
		unsafe { BSS_END },
		EntryAttributes::RW as usize,
	);
	pgtable.id_map_range(
		unsafe { KERNEL_STACK_START },
		unsafe { KERNEL_STACK_END },
		EntryAttributes::RW as usize,
	);
	pgtable.map(
		UART_BASE_ADDR,
		UART_BASE_ADDR,
		EntryAttributes::RW as usize,
		0,
	);
	pgtable.walk();
	pgtable.id_map_range(
		unsafe { HEAP_START },
		unsafe { HEAP_START + HEAP_SIZE },
		EntryAttributes::RW as usize,
	);
	use uart::*;
	/* TODO: use Rust primitives */
	let root_ppn = (&mut *pgtable as *mut Table as usize) >> 12;
	let satp_val = 8 << 60 | root_ppn;
	satp_val
}

#[no_mangle]
extern "C" fn kmain() -> usize {
	use uart::*;
	use constant::*;
	println!("Now in supervisor mode!");
	println!("Try writing to UART...");
	
	let ptr = alloc::ALLOC.lock().allocate(64 * 4096);
	let ptr = alloc::ALLOC.lock().allocate(1);
	let ptr2 = alloc::ALLOC.lock().allocate(1);
	let ptr = alloc::ALLOC.lock().allocate(1);
	alloc::ALLOC.lock().deallocate(ptr);
	let ptr = alloc::ALLOC.lock().allocate(1);
	let ptr = alloc::ALLOC.lock().allocate(1);
	let ptr = alloc::ALLOC.lock().allocate(1);
	alloc::ALLOC.lock().deallocate(ptr2);
	
	alloc::ALLOC.lock().debug();
	loop { unsafe { asm::wfi(); } }
}