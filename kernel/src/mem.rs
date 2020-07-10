// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Allocator implementation

use core::ops::Range;
use crate::info;
use crate::{println, panic};
use crate::symbols::*;
use crate::spinlock::Mutex;
use crate::page::EntryAttributes;
use crate::page::{Table, KERNEL_PGTABLE};
use crate::uart::UART_BASE_ADDR;
use crate::process::*;
use riscv::{register::*, asm};
use crate::mem;
use crate::arch;


/// Maximum number of pages. As QEMU and linker script `kernel.ld`
/// are set to have 128MB of RAM, maximum number of pages can be calculated.
pub const MAX_PAGE: usize = 128 * 1024 * 1024 / (1 << 12);

/// Frame allocator gives out one or more pages.
pub struct Allocator {
    /// A bool array records whether a page is handed out
    pub page_allocated: [usize; MAX_PAGE],
    /// Pages are handed out from `base_addr`, which is the start address
    /// of HEAP.
    pub base_addr: usize,
}

/// Align an address to upper bound according to specified order.
pub const fn align_val(val: usize, order: usize) -> usize {
    let o = (1usize << order) - 1;
    (val + o) & !o
}

/// Align an address to lower bound according to specified order.
pub const fn align_val_down(val: usize, order: usize) -> usize {
    val & !((1usize << order) - 1)
}

/// Align an address to the begin of a page.
pub const fn page_down(val: usize) -> usize {
    align_val_down(val, PAGE_ORDER)
}

impl Allocator {
    /// Returns a new allocator instance
    /// 
    /// `base_addr` should be intialized later.
    pub const fn new() -> Self {
        Allocator {
            base_addr: 0,
            page_allocated: [0; MAX_PAGE],
        }
    }

    fn offset_addr_of(&self, id: usize) -> usize {
        let addr = self.base_addr + id * PAGE_SIZE;
        addr
    }

    unsafe fn offset_id_of(&self, id: usize) -> *mut u8 {
        self.offset_addr_of(id) as *mut u8
    }

    fn offset_page_of(&self, page: *mut u8) -> usize {
        let id = (page as usize - self.base_addr) / PAGE_SIZE;
        id
    }

    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        let page_required = align_val(size, PAGE_ORDER) / PAGE_SIZE;
        for i in 0..MAX_PAGE {
            if self.page_allocated[i] == 0 {
                let mut found = true;
                for j in 0..page_required {
                    if self.page_allocated[i + j] != 0 {
                        found = false;
                        break;
                    }
                }
                if found {
                    for j in 0..page_required {
                        self.page_allocated[i + j] = page_required;
                    }
                    unsafe { return self.offset_id_of(i); }
                }
            }
        }
        panic!("no available page")
    }

    pub fn deallocate(&mut self, addr: *mut u8) {
        let id = self.offset_page_of(addr);
        let page_stride = self.page_allocated[id];
        for j in 0..page_stride {
            self.page_allocated[j + id] = 0;
        }
    }

    /// Print page allocation status
    pub fn debug(&self) {
        let mut j = 0;
        loop {
            let size = self.page_allocated[j];
            let addr = &self.page_allocated as *const usize;
            let addr = unsafe { addr.add(j) };
            if size != 0 {
                let from = self.offset_addr_of(j);
                let to = self.offset_addr_of(j + size);
                println!("{} {:X} {:X}-{:X} (pages: {:X})", j, addr as usize, from, to, size);
                j += size;
            } else {
                j += 1;
            }
            if j == MAX_PAGE {
                break;
            }
        }
    }
}

static __ALLOC: Mutex<Allocator> = Mutex::new(Allocator::new(), "global allocator");


/// Initialize allocator and kernel page table
/// This function should only be called in boot hart
pub unsafe fn init() {
    // Initialize allocator
    ALLOC().get().base_addr = align_val(HEAP_START(), PAGE_ORDER);

    // workaround for non-zero data region
    let mut alloc = ALLOC().get();
    for i in 0..MAX_PAGE {
        alloc.page_allocated[i] = 0;
    }

    let pgtable: &mut Table = &mut *(&KERNEL_PGTABLE as *const _ as *mut _); // to bypass mut ref
    pgtable.id_map_range(
        TEXT_START(),
        TEXT_END(),
        EntryAttributes::RX as usize,
    );
    pgtable.id_map_range(
        RODATA_START(),
        RODATA_END(),
        EntryAttributes::RX as usize,
    );
    pgtable.id_map_range(
        DATA_START(),
        DATA_END(),
        EntryAttributes::RW as usize,
    );
    pgtable.id_map_range(
        BSS_START(),
        BSS_END(),
        EntryAttributes::RW as usize,
    );
    pgtable.id_map_range(
        KERNEL_STACK_START(),
        KERNEL_STACK_END(),
        EntryAttributes::RW as usize,
    );
    pgtable.kernel_map(
        UART_BASE_ADDR,
        UART_BASE_ADDR,
        EntryAttributes::RW as usize,
    );
    pgtable.kernel_map(
        VIRTIO_MMIO_BASE,
        VIRTIO_MMIO_BASE,
        EntryAttributes::RW as usize,
    );
    pgtable.kernel_map(
        TRAMPOLINE_START,
        TRAMPOLINE_TEXT_START(),
        EntryAttributes::RX as usize,
    );
    pgtable.id_map_range(
        HEAP_START(),
        HEAP_START() + HEAP_SIZE(),
        EntryAttributes::RW as usize,
    );
    // CLINT
    pgtable.id_map_range(CLINT_BASE, CLINT_BASE + 0x10000, EntryAttributes::RW as usize);
    // PLIC
    pgtable.id_map_range(PLIC_BASE, PLIC_BASE + 0x400000, EntryAttributes::RW as usize);
}

pub fn hartinit() {
    let root_ppn = &KERNEL_PGTABLE as *const Table as usize;
    let satp_val = arch::build_satp(8, 0, root_ppn);
    unsafe {
        llvm_asm!("csrw satp, $0" :: "r"(satp_val));
        asm::sfence_vma(0, 0);
    }
}

#[allow(non_snake_case)]
pub fn ALLOC() -> &'static Mutex<Allocator> { &__ALLOC }

use core::alloc::{GlobalAlloc, Layout};
use crate::plic::PLIC_BASE;
use crate::clint::CLINT_BASE;
use crate::arch::hart_id;
use crate::process::my_cpu;
use crate::virtio::VIRTIO_MMIO_BASE;

struct OsAllocator {}

unsafe impl GlobalAlloc for OsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC().lock().allocate(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ALLOC().lock().deallocate(ptr);
    }
}

#[global_allocator]
static GA: OsAllocator = OsAllocator {};

#[alloc_error_handler]
pub fn alloc_error(l: Layout) -> ! {
    panic!(
        "Allocator failed to allocate {} bytes with {}-byte alignment.",
        l.size(),
        l.align()
    );
}

pub unsafe fn zero_volatile<T>(range: Range<*mut T>)
    where
        T: From<u8>,
{
    let mut ptr = range.start;
    info!("{:?}", range);
    while ptr < range.end {
        core::ptr::write_volatile(ptr, T::from(0));
        ptr = ptr.offset(1);
    }
}

pub fn debug() {
    for i in 0x8004f000 as u64..0x80093058 {
        let d = unsafe { core::ptr::read(i as *const u8) };
        if d != 0 {
            println!("0x{:x}: {:x}", i, d);
        }
        if i % 0x100000 == 0 {
            println!("{:x}", i);
        }
    }
}

pub fn alloc_stack() -> *mut u8 {
    ALLOC().lock().allocate(PAGE_SIZE * 1024)
}
