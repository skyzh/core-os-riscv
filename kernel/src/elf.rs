// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! ELF parsing

use crate::panic;
use crate::mem;
use crate::arch;
use crate::page;
use crate::process;
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

pub fn parse_elf(a: &[u8], pgtable: &mut page::Table) -> u64 {
    let a = a.as_ptr();
    /* TODO: Use something safer */
    // peek head of byte array to get ELF information
    let elfhdr = unsafe { &*(a as *const ELFHeader) };
    if elfhdr.magic != ELF_MAGIC {
        panic!("wrong magic number");
    }
    let mut proghdr = unsafe {
        let offset_u8 = a.offset(elfhdr.phoff as isize);
        offset_u8 as *const ProgramHeader
    };
    for _i in 0..elfhdr.phnum {
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
        if hdr.vaddr as usize % PAGE_SIZE != 0 {
            println!("{:X}", hdr.vaddr);
            panic!("bad elf: vaddr align")
        }
        load_segment(
            pgtable,
            hdr.vaddr as usize,
            a,
            hdr.off as usize,
            hdr.filesz as usize,
        );
        /* println!(
            "map segment ELF 0x{:X}~0x{:X} -> MEM 0x{:X}",
            hdr.off,
            hdr.off + hdr.filesz,
            hdr.vaddr
        ); */
    }
    elfhdr.entry
}

fn load_segment(
    pgtable: &mut page::Table,
    vaddr: usize,
    elf: *const u8,
    offset: usize,
    sz: usize,
) {
    let num_pages = mem::align_val(sz, PAGE_ORDER) / PAGE_SIZE;
    for i in 0..num_pages {
        let mut seg = page::Page::new();
        let src = elf as *const u8;
        unsafe {
            let src = src.add(offset + i * PAGE_SIZE);
            core::ptr::copy(src, seg.data.as_mut_ptr(), PAGE_SIZE);
        }
        use page::EntryAttributes;
        pgtable.map(
            vaddr + i * PAGE_SIZE,
            seg,
            EntryAttributes::URX as usize
        );
    }
}
