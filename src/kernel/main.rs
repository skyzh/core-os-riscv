// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![no_std]
#![no_main]
#![feature(panic_info_message, asm)]
#![feature(global_asm)]
#![feature(format_args_nl)]
#![feature(const_generics)]

global_asm!(include_str!("asm/trap.S"));
global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/symbols.S"));
global_asm!(include_str!("asm/trampoline.S"));

mod alloc;
mod arch;
mod symbols;
mod cpu;
mod elf;
mod memory;
mod page;
mod nulllock;
mod print;
mod trap;
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
extern "C" fn kinit() {
	// memory::zero_volatile(symbols::bss_range());
	uart::UART.lock().init();
	info!("Booting core-os...");
	info!("Drivers:");
	info!("  UART intialized");
	info!("Booting on hart {}", mhartid::read());

	use symbols::*;
	/*
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
	*/
	use page::EntryAttributes;
	use page::{Table, KERNEL_PGTABLE};
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
	pgtable.map(
		TRAMPOLINE_START,
		unsafe { TRAMPOLINE_TEXT_START },
		page::EntryAttributes::RX as usize,
		0
	);
	pgtable.id_map_range(
		unsafe { HEAP_START },
		unsafe { HEAP_START + HEAP_SIZE },
		EntryAttributes::RW as usize,
	);
	// CLINT
	//  -> MSIP
	pgtable.id_map_range(0x0200_0000, 0x0200_ffff, EntryAttributes::RW as usize);
	// PLIC
	pgtable.id_map_range(0x0c00_0000, 0x0c00_2000, EntryAttributes::RW as usize);
	pgtable.id_map_range(0x0c20_0000, 0x0c20_8000, EntryAttributes::RW as usize);
	use uart::*;
	/* TODO: use Rust primitives */
	unsafe {
		let root_ppn = &mut *pgtable as *mut Table as usize;
		let satp_val = cpu::build_satp(8, 0, root_ppn);
		mscratch::write(&mut cpu::KERNEL_TRAP_FRAME[0] as *mut cpu::TrapFrame as usize);
		cpu::KERNEL_TRAP_FRAME[0].satp = satp_val;
		let stack_addr = alloc::ALLOC.lock().allocate(1);
		cpu::KERNEL_TRAP_FRAME[0].sp = stack_addr.add(alloc::PAGE_SIZE);
		cpu::KERNEL_TRAP_FRAME[0].hartid = 0;
		pgtable.id_map_range(
			stack_addr as usize,
			stack_addr.add(alloc::PAGE_SIZE) as usize,
			EntryAttributes::RW as usize,
		);
		unsafe {
			asm!("csrw satp, $0" :: "r"(satp_val));
			asm!("sfence.vma zero, zero");
		}
	}
	info!("Page table set up, switching to supervisor mode");
}

#[no_mangle]
extern "C" fn kinit_hart(hartid: usize) {
	// All non-0 harts initialize here.
	unsafe {
		// We have to store the kernel's table. The tables will be moved
		// back and forth between the kernel's table and user
		// applicatons' tables.
		mscratch::write(&mut cpu::KERNEL_TRAP_FRAME[hartid] as *mut cpu::TrapFrame as usize);
		cpu::KERNEL_TRAP_FRAME[hartid].hartid = hartid;
		// We can't do the following until zalloc() is locked, but we
		// don't have locks, yet :( cpu::KERNEL_TRAP_FRAME[hartid].satp
		// = cpu::KERNEL_TRAP_FRAME[0].satp;
		// cpu::KERNEL_TRAP_FRAME[hartid].trap_stack = page::zalloc(1);
	}
	// info!("{} initialized", hartid);
}

pub fn wait_forever() -> ! {
	loop {
		unsafe {
			asm::wfi();
		}
	}
}

static USER_PROGRAM: &'static [u8; 9104] = include_bytes!("../../target/riscv64gc-unknown-none-elf/release/loop");

#[no_mangle]
extern "C" fn kmain() -> ! {
	use symbols::*;
	use uart::*;
	info!("Now in supervisor mode");
	/*
	unsafe {
		let mtimecmp = 0x0200_4000 as *mut u64;
		let mtime = 0x0200_bff8 as *const u64;
		// The frequency given by QEMU is 10_000_000 Hz, so this sets
		// the next interrupt to fire one second from now.
		mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
	}*/
	info!("entering user program...");
	elf::run_elf(USER_PROGRAM);
	wait_forever();
}

pub fn test_alloc() {
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
}
