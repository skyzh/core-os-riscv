use crate::alloc;
use crate::mmu;
use crate::{info, print, println};

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

pub fn run_elf<const N: usize>(a: &'static [u8; N]) {
    /* TODO: Use something safer */
    // peek head of byte array to get ELF information
    let mut pagetable = mmu::Table::new();
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
        info!("mapping segment...");
        load_segment(
            &mut pagetable,
            hdr.vaddr as usize,
            a,
            hdr.off as usize,
            hdr.filesz as usize,
        );
    }
    info!("elf loaded");
}

fn load_segment<const N: usize>(
    pagetable: &mut mmu::Table,
    vaddr: usize,
    elf: &'static [u8; N],
    offset: usize,
    sz: usize,
) {
    let num_pages = sz / alloc::PAGE_SIZE;
    let mut alc = alloc::ALLOC.lock();
    for i in 0..num_pages {
        let seg = alc.allocate(alloc::PAGE_SIZE);
        let src = elf as *const u8;
        unsafe {
            let src = src.add(offset + i * alloc::PAGE_SIZE);
            core::ptr::copy(src, seg, alloc::PAGE_SIZE);
        }
    }
}
