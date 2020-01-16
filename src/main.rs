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
pub unsafe extern "C" fn start() -> ! {
	memory::zero_volatile(constant::bss_range());
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
extern "C" fn kmain() {
	uart::UART.lock().init();

	println!("mhartid {}", mhartid::read());

	/*
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
	*/
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
	use mmu::{KERNEL_PGTABLE, Table};
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
	pgtable.walk();
	pgtable.id_map_range(
		unsafe { HEAP_START },
		unsafe { HEAP_START + HEAP_SIZE },
		EntryAttributes::RW as usize,
	);
	/* TODO: use Rust primitives */
	let root_ppn = (&mut *pgtable as *mut Table as usize) >> 12;
	let satp_val = 8 << 60 | root_ppn;
	unsafe {
		asm!("csrw satp, $0" :: "r"(satp_val));
		asm!("sfence.vma");
	}
	println!("MMU intialized!");
	loop {}
}
