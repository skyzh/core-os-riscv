// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::alloc::{self, ALLOC};
use crate::nulllock::Mutex;
use crate::{print, println, panic};
use crate::symbols::*;

const TABLE_ENTRY_CNT: usize = 512;

#[repr(C)]
#[repr(align(4096))]
pub struct Table {
    pub entries: [Entry; TABLE_ENTRY_CNT],
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Entry(usize);

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct VPN(usize);

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PPN(usize);

pub enum EntryAttributes {
    D = 1 << 7,
    A = 1 << 6,
    G = 1 << 5,
    U = 1 << 4,
    X = 1 << 3,
    W = 1 << 2,
    R = 1 << 1,
    V = 1 << 0,
    RW = 0b11 << 1,
    RX = 0b101 << 1,
    UR = 0b10010,
    URW = 0b10110,
    URX = 0b11010,
}

impl Entry {
    pub fn is_d(&self) -> bool {
        self.0 & EntryAttributes::D as usize != 0
    }
    pub fn is_a(&self) -> bool {
        self.0 & EntryAttributes::A as usize != 0
    }
    pub fn is_g(&self) -> bool {
        self.0 & EntryAttributes::G as usize != 0
    }
    pub fn is_u(&self) -> bool {
        self.0 & EntryAttributes::U as usize != 0
    }
    pub fn is_x(&self) -> bool {
        self.0 & EntryAttributes::X as usize != 0
    }
    pub fn is_w(&self) -> bool {
        self.0 & EntryAttributes::W as usize != 0
    }
    pub fn is_r(&self) -> bool {
        self.0 & EntryAttributes::R as usize != 0
    }
    pub fn is_v(&self) -> bool {
        self.0 & EntryAttributes::V as usize != 0
    }
    pub fn is_leaf(&self) -> bool {
        self.0 & 0xe != 0
    }
    pub fn paddr(&self) -> PPN {
        PPN((self.0 & !0x3ff) << 2)
    }
    pub const fn new(ppn: usize, flags: usize) -> Self {
        Self(((ppn & !0xfff) >> 2) | flags)
    }
}

impl PPN {
    pub fn ppn0(&self) -> usize {
        (self.0 >> 12) & 0x1ff
    }
    pub fn ppn1(&self) -> usize {
        (self.0 >> 21) & 0x1ff
    }
    pub fn ppn2(&self) -> usize {
        (self.0 >> 30) & 0x3ff_ffff
    }
    pub fn idx(&self, id: usize) -> usize {
        match id {
            0 => self.ppn0(),
            1 => self.ppn1(),
            2 => self.ppn2(),
            _ => unreachable!(),
        }
    }
}

impl VPN {
    pub fn vpn0(&self) -> usize {
        (self.0 >> 12) & 0x1ff
    }
    pub fn vpn1(&self) -> usize {
        (self.0 >> 21) & 0x1ff
    }
    pub fn vpn2(&self) -> usize {
        (self.0 >> 30) & 0x1ff
    }
    pub fn idx(&self, id: usize) -> usize {
        match id {
            0 => self.vpn0(),
            1 => self.vpn1(),
            2 => self.vpn2(),
            _ => unreachable!(),
        }
    }
}

impl Table {
    pub const fn new() -> Self {
        Table {
            entries: [Entry(0); TABLE_ENTRY_CNT],
        }
    }

    pub const fn len(&self) -> usize {
        TABLE_ENTRY_CNT
    }

    pub fn map(&mut self, vaddr: usize, paddr: usize, flags: usize, level: usize) {
        if paddr % PAGE_SIZE != 0 {
            panic!("paddr {:x} not aligned", paddr);
        }
        if vaddr % PAGE_SIZE != 0 {
            panic!("vaddr {:x} not aligned", vaddr);
        }
        let mut alloc = ALLOC().lock();
        let vpn = VPN(vaddr);
        let mut v = &mut self.entries[vpn.vpn2()];
        for lvl in (level..2).rev() {
            if !v.is_v() {
                let page = alloc.allocate(1);
                *v = Entry::new(page as usize, EntryAttributes::V as usize);
            }
            let entry = v.paddr().0 as *mut Entry;
            v = unsafe { entry.add(vpn.idx(lvl)).as_mut().unwrap() };
        }
        *v = Entry::new(paddr, flags | EntryAttributes::V as usize)
    }

    pub fn paddr_of(&self, vaddr: usize) -> Option<usize> {
        let vpn = VPN(vaddr);
        let mut v = &self.entries[vpn.vpn2()];
        if vaddr % PAGE_SIZE != 0 {
            panic!("vaddr {:x} not aligned", vaddr);
        }
        for lvl in (0..2).rev() {
            if !v.is_v() {
                return None;
            }
            let entry = v.paddr().0 as *mut Entry;
            v = unsafe { entry.add(vpn.idx(lvl)).as_mut().unwrap() };
        }
        Some(v.paddr().0)
    }

    fn _walk(&self, level: usize, vpn: usize) {
        for i in 0..self.len() {
            let v = &self.entries[i];
            if v.is_v() {
                for j in 0..(2 - level) {
                    print!(".");
                }
                if !v.is_leaf() {
                    println!(
                        "{}: 0x{:X} -> 0x{:X}",
                        i,
                        (vpn << 9 | i) << (9 * level + 12),
                        v.paddr().0
                    );
                    let table = v.paddr().0 as *const Table;
                    let table = unsafe { table.as_ref().unwrap() };
                    table._walk(level - 1, (vpn << 9) | i);
                } else {
                    println!("{}: 0x{:X} -> 0x{:X}", i, (vpn << 9 | i) << 12, v.paddr().0);
                }
            }
        }
    }
    pub fn walk(&self) {
        self._walk(2, 0);
    }

    pub fn id_map_range(&mut self, start: usize, end: usize, bits: usize) {
        let mut memaddr = start & !(PAGE_SIZE - 1);
        let num_kb_pages = (alloc::align_val(end, 12) - memaddr) / PAGE_SIZE;

        for _ in 0..num_kb_pages {
            self.map(memaddr, memaddr, bits, 0);
            memaddr += 1 << 12;
        }
    }

    pub fn map_range(&mut self, start: usize, end: usize, vaddr_start: usize, bits: usize) {
        let mut memaddr = start & !(PAGE_SIZE - 1);
        let mut vaddr_start = vaddr_start & !(PAGE_SIZE - 1);
        let num_kb_pages = (alloc::align_val(end, 12) - memaddr) / PAGE_SIZE;

        for _ in 0..num_kb_pages {
            self.map(vaddr_start, memaddr, bits, 0);
            memaddr += 1 << 12;
            vaddr_start += 1 << 12;
        }
    }
}

static mut __KERNEL_PGTABLE: Mutex<Table> =  Mutex::new(Table::new());

pub fn init() {
}

pub fn KERNEL_PGTABLE() -> &'static mut Mutex<Table> { unsafe { &mut __KERNEL_PGTABLE } }
