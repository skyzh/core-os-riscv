// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! virt-io driver

use crate::spinlock::Mutex;
use crate::virtio::VIRTIO_MMIO::{MAGIC_VALUE, VERSION, DEVICE_ID, VENDOR_ID, STATUS, DEVICE_FEATURES, GUEST_PAGE_SIZE, QUEUE_SEL, QUEUE_NUM_MAX, QUEUE_NUM, QUEUE_PFN};
use crate::panic;
use crate::virtio::VIRTIO_CONFIG_S::{ACKNOWLDGE, DRIVER, FEATURES_OK, DRIVER_OK};
use crate::virtio::VIRTIO_FEATURE::{BLK_F_RO, BLK_F_SCSI, BLK_F_CONFIG_WCE, BLK_F_MQ, F_ANY_LAYOUT, RING_F_EVENT_IDX, RING_F_INDIRECT_DESC};
use crate::symbols::{PAGE_SIZE, PAGE_ORDER};

pub const VIRTIO_MMIO_BASE: usize = 0x10001000;

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
    pub const fn val(self) -> usize {
        self as usize + VIRTIO_MMIO_BASE
    }
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

pub const VRING_DESC_F_NEXT: usize = 1;
pub const VRING_DESC_F_WRITE: usize = 2;

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

pub const VIRTIO_BLK_T_IN: usize = 0;
pub const VIRTIO_BLK_T_OUT: usize = 1;

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

const AVAIL_SZ: usize = (PAGE_SIZE - DESC_NUM * core::mem::size_of::<VRingDesc>()) / core::mem::size_of::<u16>();

#[repr(C)]
#[repr(align(4096))]
pub struct VirtIO {
    pub desc: [VRingDesc; DESC_NUM],
    pub avail: [u16; AVAIL_SZ],
    pub used: [UsedArea; DESC_NUM],
    pub free: [bool; DESC_NUM],
    pub used_idx: u16,
}

impl VirtIO {
    pub const fn new() -> Self {
        Self {
            desc: [VRingDesc::new(); DESC_NUM],
            avail: [0; AVAIL_SZ],
            used: [UsedArea::new(); DESC_NUM],
            free: [false; DESC_NUM],
            used_idx: 0,
        }
    }

    pub unsafe fn init(&mut self) {
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

        QUEUE_PFN.ptr().write_volatile(((self as *mut _ as usize) >> PAGE_ORDER) as u32);

        for i in 0..DESC_NUM {
            self.free[i] = true;
        }
    }
    
    fn alloc_desc(&mut self) -> Option<usize> {
        for i in 0..DESC_NUM {
            if self.free[i] {
                self.free[i] = false;
                return Some(i);
            }
        }
        None
    }
    
    fn free_desc(&mut self, i: usize) {
        if i >= DESC_NUM {
            panic!("invalid desc");
        }
        if self.free[i] {
            panic!("already free");
        }
        self.desc[i].addr = 0;
        self.free[i] = true;
    }
}

/// VirtIO driver object
static __VIRTIO: Mutex<VirtIO> = Mutex::new(VirtIO::new(), "virtio");

/// Global function to get an instance of VirtIO driver
#[allow(non_snake_case)]
pub fn VIRTIO() -> &'static Mutex<VirtIO> { &__VIRTIO }

pub unsafe fn init() {
    VIRTIO().get().init();
}


/// virtual io interrupt
pub fn virtiointr() {}

pub mod tests {
    use super::*;

    pub fn tests() -> &'static [(&'static str, fn())] {
        &[("memory layout", test_memory_layout)]
    }

    /// Test virtio memory layout
    pub fn test_memory_layout() {
        let virtio = VIRTIO().lock();
        assert_eq!(&virtio.desc as *const _ as usize % PAGE_SIZE, 0);
        assert_eq!(&virtio.used as *const _ as usize % PAGE_SIZE, 0);
        assert_eq!(&virtio.used as *const _ as usize - &virtio.desc as *const _ as usize, PAGE_SIZE);
    }
}
