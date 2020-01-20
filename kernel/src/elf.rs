// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::alloc;
use crate::cpu::{self, TrapFrame};
use crate::page;
use crate::symbols::*;
use crate::{info, println};

#[repr(C)]
pub struct ELFHeader {
    pub magic: u32,
    pub elf: [u8; 12],
    pub etype: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C)]
pub struct ProgramHeader {
    pub ptype: u32,
    pub flags: u32,
    pub off: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

const ELF_PROG_LOAD: u32 = 1;
const ELF_PROG_FLAG_EXEC: u32 = 1;
const ELF_PROG_FLAG_WRITE: u32 = 2;
const ELF_PROG_FLAG_READ: u32 = 4;
const ELF_MAGIC: u32 = 0x464C457F;

fn trampoline(tf: usize, satp_val: usize) {
    let uservec_offset = userret as usize - unsafe { TRAMPOLINE_TEXT_START };
    let fn_addr = unsafe { (TRAMPOLINE_START + uservec_offset) as *const () };
    let fn_addr: extern "C" fn(usize, usize) -> usize = unsafe { core::mem::transmute(fn_addr) };
    (fn_addr)(tf, satp_val);
}

pub fn run_elf<const N: usize>(a: &'static [u8; N]) {
    /* TODO: Use something safer */
    // peek head of byte array to get ELF information
    let mut pgtable = page::Table::new();
    let elfhdr = &unsafe { core::mem::transmute::<&[u8], &[ELFHeader]>(a) }[0];
    if elfhdr.magic != ELF_MAGIC {
        info!("wrong magic number");
        return;
    }
    let mut proghdr = unsafe {
        let offset_u8 = (&a[0] as *const u8).offset(elfhdr.phoff as isize);
        offset_u8 as *const ProgramHeader
    };
    for i in 0..elfhdr.phnum {
        let hdr: &ProgramHeader = unsafe {
            let hdr = &*proghdr;
            proghdr = proghdr.offset(1);
            hdr
        };
        if hdr.ptype != ELF_PROG_LOAD {
            continue;
        }
        if hdr.memsz < hdr.filesz {
            panic!("bad elf: memsz");
        }
        if hdr.vaddr + hdr.memsz < hdr.vaddr {
            panic!("bad elf: vaddr");
        }
        if hdr.vaddr as usize % alloc::PAGE_SIZE != 0 {
            println!("{:X}", hdr.vaddr);
            panic!("bad elf: vaddr align")
        }
        load_segment(
            &mut pgtable,
            hdr.vaddr as usize,
            a,
            hdr.off as usize,
            hdr.filesz as usize,
        );
        info!(
            "map segment ELF 0x{:X}~0x{:X} -> MEM 0x{:X}",
            hdr.off,
            hdr.off + hdr.filesz,
            hdr.vaddr
        );
    }
    info!("elf loaded");
    let mut tf = TrapFrame::zero();
    // map trampoline
    pgtable.map(
        TRAMPOLINE_START,
        unsafe { TRAMPOLINE_TEXT_START },
        page::EntryAttributes::RX as usize,
        0,
    );
    // map trapframe
    pgtable.map(
        TRAPFRAME_START,
        &tf as *const _ as usize,
        page::EntryAttributes::RW as usize,
        0,
    );
    // map user stack
    let seg = alloc::ALLOC().lock().allocate(alloc::PAGE_SIZE);
    let stack_begin = 0x80001000;
    pgtable.map(
        stack_begin,
        seg as usize,
        page::EntryAttributes::URW as usize,
        0,
    );
    pgtable.walk();

    tf.epc = 0x0000000080000000;

    unsafe {
        stvec::write(
            (uservec as usize - TRAMPOLINE_TEXT_START) + TRAMPOLINE_START,
            stvec::TrapMode::Direct,
        );
    }
    tf.satp = unsafe { cpu::KERNEL_TRAP_FRAME[0].satp };
    tf.sp = unsafe { cpu::KERNEL_TRAP_FRAME[0].sp };
    tf.trap = crate::trap::usertrap as usize;
    tf.hartid = unsafe { cpu::KERNEL_TRAP_FRAME[0].hartid };

    sepc::write(tf.epc);
    tf.regs[2] = stack_begin + 0x1000; // sp
    unsafe { cpu::intr_off(); }
    use riscv::register::*;
    unsafe {
        sstatus::set_spie();
        sstatus::set_spp(sstatus::SPP::User);
    }

    let root_ppn = &mut pgtable as *mut page::Table as usize;
    let satp_val = crate::cpu::build_satp(8, 0, root_ppn);
    println!("jumping to trampoline...");
    trampoline(TRAPFRAME_START, satp_val);
}

fn load_segment<const N: usize>(
    pgtable: &mut page::Table,
    vaddr: usize,
    elf: &'static [u8; N],
    offset: usize,
    sz: usize,
) {
    let num_pages = alloc::align_val(sz, PAGE_ORDER) / PAGE_SIZE;
    let mut alc = alloc::ALLOC().lock();
    for i in 0..num_pages {
        let seg = alc.allocate(alloc::PAGE_SIZE);
        let src = elf as *const u8;
        unsafe {
            let src = src.add(offset + i * alloc::PAGE_SIZE);
            core::ptr::copy(src, seg, alloc::PAGE_SIZE);
        }
        use page::EntryAttributes;
        pgtable.map(
            vaddr + i * alloc::PAGE_SIZE,
            seg as usize,
            EntryAttributes::URX as usize,
            0,
        );
        /*
        for i in 0..0x20 {
            unsafe { print!("{:X}", core::ptr::read(src.add(i))); }
        }
        println!(""); */
    }
}
