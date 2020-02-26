// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! virt-io driver

use crate::spinlock::{Mutex, MutexGuard};
use crate::{panic, info};
use crate::symbols::{PAGE_SIZE, PAGE_ORDER};
use crate::process::{wakeup, sleep};
use alloc::boxed::Box;
use crate::arch::__sync_synchronize;
use crate::uart::UART;
use core::sync::atomic::Ordering;

/// VIRTIO base address on QEMU RISC-V
pub const VIRTIO_MMIO_BASE: usize = 0x10001000;

/// VIRTIO MMIO address offset
#[allow(non_camel_case_types)]
pub enum VIRTIO_MMIO {
    MAGIC_VALUE = 0x0,
    VERSION = 0x4,
    DEVICE_ID = 0x8,
    VENDOR_ID = 0xc,
    DEVICE_FEATURES = 0x10,
    DRIVER_FEATURES = 0x20,
    GUEST_PAGE_SIZE = 0x28,
    QUEUE_SEL = 0x30,
    QUEUE_NUM_MAX = 0x34,
    QUEUE_NUM = 0x38,
    QUEUE_ALIGN = 0x3c,
    QUEUE_PFN = 0x40,
    QUEUE_READY = 0x44,
    QUEUE_NOTIFY = 0x50,
    INTERRUPT_STATUS = 0x60,
    INTERRUPT_ACK = 0x64,
    STATUS = 0x70,
}

impl VIRTIO_MMIO {
    /// Get address of MMIO from enum
    pub const fn val(self) -> usize {
        self as usize + VIRTIO_MMIO_BASE
    }
    /// Get pointer to MMIO register from enum
    pub const fn ptr(self) -> *mut u32 {
        self.val() as _
    }
}

#[allow(non_camel_case_types)]
pub enum VIRTIO_CONFIG_S {
    ACKNOWLDGE = 1,
    DRIVER = 1 << 1,
    DRIVER_OK = 1 << 2,
    FEATURES_OK = 1 << 3,
}

impl VIRTIO_CONFIG_S {
    pub const fn val(self) -> u32 { self as _ }
}

#[allow(non_camel_case_types)]
pub enum VIRTIO_FEATURE {
    BLK_F_RO = 5,
    BLK_F_SCSI = 7,
    BLK_F_CONFIG_WCE = 11,
    BLK_F_MQ = 12,
    F_ANY_LAYOUT = 27,
    RING_F_INDIRECT_DESC = 28,
    RING_F_EVENT_IDX = 29,
}

impl VIRTIO_FEATURE {
    pub fn bit(self) -> u32 {
        (1 << self as usize) as u32
    }
}

pub const DESC_NUM: usize = 8;

#[repr(C)]
pub struct VRingDesc {
    pub addr: usize,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl VRingDesc {
    pub const fn new() -> Self {
        Self {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

pub const VRING_DESC_F_NEXT: u16 = 1;
pub const VRING_DESC_F_WRITE: u16 = 2;

#[repr(C)]
pub struct VRingUsedElem {
    pub id: u32,
    pub len: u32,
}

impl VRingUsedElem {
    pub const fn new() -> Self {
        Self {
            id: 0,
            len: 0,
        }
    }
}

pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;

#[repr(C)]
pub struct UsedArea {
    pub flags: u16,
    pub id: u16,
    pub elems: [VRingUsedElem; DESC_NUM],
}

impl UsedArea {
    pub const fn new() -> Self {
        Self {
            flags: 0,
            id: 0,
            elems: [VRingUsedElem::new(); DESC_NUM],
        }
    }
}

pub struct InflightOp {
    pub buf: Box<Buf>,
    pub status: u8,
}

/// Size of avail array
const AVAIL_SZ: usize = (PAGE_SIZE - DESC_NUM * core::mem::size_of::<VRingDesc>()) / core::mem::size_of::<u16>();

#[repr(C)]
#[repr(align(4096))]
pub struct VirtIOData {
    /// VIRTIO MMIO descriptor register
    pub desc: [VRingDesc; DESC_NUM],
    /// VIRTIO MMIO descriptor avail register (padding to page size)
    pub avail: [u16; AVAIL_SZ],
    /// VIRTIO MMIO descriptor used register
    pub used: [UsedArea; DESC_NUM],

    /// is descriptor free
    pub free: [bool; DESC_NUM],
    /// used index of used array
    pub used_idx: u16,
    /// in-flight operations
    pub info: [Option<InflightOp>; DESC_NUM],
}

pub struct VirtIO(Mutex<VirtIOData>);

/// VIRTIO buffer size
pub const BSIZE: usize = 1024;

/// VIRTIO Buffer
#[repr(C)]
pub struct Buf {
    pub valid: bool,
    /// TODO: this can be removed
    pub disk: i32,
    /// device ID
    pub dev: u32,
    /// block number
    pub blockno: u32,
    /// buffer data
    pub data: [u8; BSIZE],
}

impl Buf {
    pub const fn new() -> Self {
        Self {
            valid: false,
            disk: 0,
            dev: 0,
            blockno: 0,
            data: [0; BSIZE],
        }
    }
}

#[repr(C)]
pub struct BlkOutHdr {
    pub blk_type: u32,
    reserved: u32,
    sector: usize,
}

impl VirtIOData {
    /// Free one descriptor
    fn free_desc(&mut self, i: usize) {
        if i >= DESC_NUM {
            panic!("invalid desc");
        }
        if self.free[i] {
            panic!("already free");
        }
        self.desc[i].addr = 0;
        self.free[i] = true;
        wakeup(&self.free[0]);
    }

    /// Allocate one descriptor
    fn alloc_desc(&mut self) -> Option<usize> {
        for i in 0..DESC_NUM {
            if self.free[i] {
                self.free[i] = false;
                return Some(i);
            }
        }
        None
    }

    /// Allocate three descriptors, return array of indices
    fn alloc3_desc(&mut self) -> Option<[usize; 3]> {
        let mut idx = [0; 3];
        for i in 0..3 {
            match self.alloc_desc() {
                Some(x) => idx[i] = x,
                None => {
                    for j in 0..i {
                        self.free_desc(idx[j]);
                    }
                    return None;
                }
            }
        }
        Some(idx)
    }

    /// Free descriptor chain
    fn free_chain(&mut self, mut i: usize) {
        loop {
            self.free_desc(i);
            if self.desc[i].flags & VRING_DESC_F_NEXT != 0 {
                i = self.desc[i].next as usize;
            } else {
                break;
            }
        }
    }
}

impl VirtIO {
    pub const fn new() -> Self {
        Self(
            Mutex::new(VirtIOData {
                desc: [VRingDesc::new(); DESC_NUM],
                avail: [0; AVAIL_SZ],
                used: [UsedArea::new(); DESC_NUM],
                free: [false; DESC_NUM],
                used_idx: 0,
                info: [None; DESC_NUM],
            }, "vdisk"))
    }

    /// Initialize VIRTIO driver.
    ///
    /// Should be called in booting hart.
    pub unsafe fn init(&mut self) {
        use VIRTIO_MMIO::*;
        use VIRTIO_CONFIG_S::*;
        use VIRTIO_FEATURE::*;

        let mut vio = self.0.get();

        if MAGIC_VALUE.ptr().read_volatile() != 0x74726976 {
            panic!("cannot find virtio disk: magic value");
        }
        if VERSION.ptr().read_volatile() != 1 {
            panic!("cannot find virtio disk: version");
        }
        if DEVICE_ID.ptr().read_volatile() != 2 {
            panic!("cannot find virtio disk: device id");
        }
        if VENDOR_ID.ptr().read_volatile() != 0x554d4551 {
            panic!("cannot find virtio disk: vendor id");
        }

        let mut status: u32 = 0;
        status |= ACKNOWLDGE.val();
        STATUS.ptr().write_volatile(status);

        status |= DRIVER.val();
        STATUS.ptr().write_volatile(status);

        let mut features: u32 = DEVICE_FEATURES.ptr().read_volatile();
        features &= !BLK_F_RO.bit();
        features &= !BLK_F_SCSI.bit();
        features &= !BLK_F_CONFIG_WCE.bit();
        features &= !BLK_F_MQ.bit();
        features &= !F_ANY_LAYOUT.bit();
        features &= !RING_F_EVENT_IDX.bit();
        features &= !RING_F_INDIRECT_DESC.bit();
        DEVICE_FEATURES.ptr().write_volatile(features);

        status |= FEATURES_OK.val();
        STATUS.ptr().write_volatile(status);

        status |= DRIVER_OK.val();
        STATUS.ptr().write_volatile(status);

        GUEST_PAGE_SIZE.ptr().write_volatile(PAGE_SIZE as u32);

        QUEUE_SEL.ptr().write_volatile(0);
        let max = QUEUE_NUM_MAX.ptr().read_volatile();
        if max == 0 {
            panic!("virtio disk has no queue");
        }
        if max < DESC_NUM as u32 {
            panic!("virtio disk max queue too short {} < {}", max, DESC_NUM);
        }
        QUEUE_NUM.ptr().write_volatile(DESC_NUM as u32);

        QUEUE_PFN.ptr().write_volatile(((vio as *mut _ as usize) >> PAGE_ORDER) as u32);

        for i in 0..DESC_NUM {
            vio.free[i] = true;
        }
    }

    /// Read-write operation
    fn rw(&mut self, mut b: Box<Buf>, write: bool) -> Box<Buf> {
        use VIRTIO_MMIO::*;

        let sector = b.blockno as usize * (BSIZE / 512);

        let mut vio = self.0.lock();

        let idx: [usize; 3] = loop {
            if let Some(idx) = vio.alloc3_desc() {
                break idx;
            }
            vio = sleep(&vio.free[0] as *const _, vio);
        };

        // info!("3 desc: {:?}", idx);

        let buf0 = BlkOutHdr {
            reserved: 0,
            sector,
            blk_type: if write { VIRTIO_BLK_T_OUT } else { VIRTIO_BLK_T_IN },
        };


        // VIRTIO 5.2.6.4
        // MUST use a single 8-byte descriptor containing type, reserved and sector,
        // followed by descriptors for data, then finally a separate 1-byte descriptor for status.

        {
            let desc0 = &mut vio.desc[idx[0]];
            desc0.addr = &buf0 as *const _ as usize;
            desc0.len = core::mem::size_of::<BlkOutHdr>() as u32;
            desc0.flags = VRING_DESC_F_NEXT;
            desc0.next = idx[1] as u16;
        }

        {
            let desc1 = &mut vio.desc[idx[1]];
            desc1.addr = b.data.as_mut_ptr() as usize;
            desc1.len = BSIZE as u32;
            desc1.flags = if write { 0 } else { VRING_DESC_F_WRITE };
            desc1.flags |= VRING_DESC_F_NEXT;
            desc1.next = idx[2] as u16;
        }

        b.disk = 1;
        vio.info[idx[0]] = Some(InflightOp {
            buf: b,
            status: 0,
        });

        {
            let addr = &vio.info[idx[0]].as_ref().unwrap().status as *const _ as usize;
            {
                let desc2 = &mut vio.desc[idx[2]];

                desc2.addr = addr;
                desc2.len = 1;
                desc2.flags = VRING_DESC_F_WRITE;
                desc2.next = 0;
            }

            let idx_id = 2 + vio.avail[1] as usize % DESC_NUM;
            vio.avail[idx_id] = idx[0] as u16;

            __sync_synchronize();

            vio.avail[1] = vio.avail[1] + 1;

            unsafe { QUEUE_NOTIFY.ptr().write_volatile(0); }
            // info!("{:x}", &vio.info[idx[0]].as_ref().unwrap().status as *const _ as usize);
            let buf_addr = &*vio.info[idx[0]].as_ref().unwrap().buf as *const _;
            // info!("sleep {:x} id={}", buf_addr as usize, vio.info[idx[0]].as_ref().unwrap().status);
            while vio.info[idx[0]].as_ref().unwrap().buf.disk == 1 {
                vio = sleep(buf_addr, vio);
            }
        }
        let result = core::mem::replace(&mut vio.info[idx[0]], None);
        unsafe { vio.info[idx[0]] = core::mem::zeroed(); }
        vio.info[idx[0]] = None;
        vio.free_chain(idx[0]);
        result.unwrap().buf
    }

    /// Read from device and block number
    pub fn read(&mut self, dev: u32, blockno: u32) -> Box<Buf> {
        let mut buf = box Buf::new();
        buf.dev = dev;
        buf.blockno = blockno;
        self.rw(buf, false)
    }

    /// Write buffer to disk
    pub fn write(&mut self, buf: Box<Buf>) {
        self.rw(buf, true);
    }
}

/// VirtIO driver object
static mut __VIRTIO: VirtIO = VirtIO::new();

/// Global function to get an instance of VirtIO driver
#[allow(non_snake_case)]
pub fn VIRTIO() -> &'static mut VirtIO { unsafe { &mut __VIRTIO } }

pub unsafe fn init() {
    VIRTIO().init();
}


/// VIRTIO interrupt
pub fn virtiointr() {
    use crate::info;
    let mut disk = VIRTIO().0.lock();
    while disk.used_idx as usize % DESC_NUM != disk.used[0].id as usize % DESC_NUM {

        let id = disk.used[0].elems[disk.used_idx as usize].id as usize;

        if disk.info[id].is_none() {
            panic!("invalid id");
        }

        let info = disk.info[id].as_mut().unwrap();

        // info!("processing 0x{:x}", &*info.buf as *const _ as usize);

        if info.status != 0 {
            panic!("virtio_disk_intr status status={} id={}", info.status, id);
        }

        info.buf.disk = 0;

        wakeup(&*info.buf);

        disk.used_idx = ((disk.used_idx + 1) as usize % DESC_NUM) as u16;
    }
}

pub mod tests {
    use super::*;

    pub fn tests() -> &'static [(&'static str, fn())] {
        &[
            ("memory layout", test_memory_layout),
            ("read and write", test_rw)
        ]
    }

    /// Test virtio memory layout
    pub fn test_memory_layout() {
        let virtio = VIRTIO().0.lock();
        assert_eq!(&virtio.desc as *const _ as usize % PAGE_SIZE, 0);
        assert_eq!(&virtio.used as *const _ as usize % PAGE_SIZE, 0);
        assert_eq!(&virtio.used as *const _ as usize - &virtio.desc as *const _ as usize, PAGE_SIZE);
    }

    use crate::{print, println};

    /// Test read and write
    pub fn test_rw() {
        let virtio = VIRTIO();
        let b = virtio.read(1, 0);
        unsafe { println!("size: {}", core::ptr::read(b.data.as_ptr() as *const usize)); }
        unsafe { println!("offset: {}", core::ptr::read(b.data.as_ptr().add(8) as *const usize)); }
        for i in 16..b.data.len() {
            let d = b.data[i];
            if d == 0 {
                break;
            }
            print!("{}", b.data[i] as char);
        }
        println!();
    }
}
