// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V Platform-Level Interrupt Controller

use crate::spinlock::Mutex;
use crate::process::my_cpu;
use crate::arch::hart_id;

pub const PLIC_BASE: usize = 0x0c00_0000;
pub const PLIC_PRIORITY: usize = PLIC_BASE + 0x0;
pub const PLIC_PENDING: usize = PLIC_BASE + 0x1000;
pub const PLIC_MENABLE_BASE: usize = PLIC_BASE + 0x2000;
pub const PLIC_SENABLE_BASE: usize = PLIC_BASE + 0x2080;
pub const PLIC_MPRIORITY_BASE: usize = PLIC_BASE + 0x200000;
pub const PLIC_SPRIORITY_BASE: usize = PLIC_BASE + 0x201000;
pub const PLIC_MCLAIM_BASE: usize = PLIC_BASE + 0x200004;
pub const PLIC_SCLAIM_BASE: usize = PLIC_BASE + 0x201004;

#[allow(non_snake_case)]
pub const fn PLIC_MENABLE(hart: usize) -> usize { PLIC_MENABLE_BASE + hart * 0x100 }

#[allow(non_snake_case)]
pub const fn PLIC_SENABLE(hart: usize) -> usize { PLIC_SENABLE_BASE + hart * 0x100 }

#[allow(non_snake_case)]
pub const fn PLIC_MPRIORITY(hart: usize) -> usize { PLIC_MPRIORITY_BASE + hart * 0x2000 }

#[allow(non_snake_case)]
pub const fn PLIC_SPRIORITY(hart: usize) -> usize { PLIC_SPRIORITY_BASE + hart * 0x2000 }

#[allow(non_snake_case)]
pub const fn PLIC_MCLAIM(hart: usize) -> usize { PLIC_MCLAIM_BASE + hart * 0x2000 }

#[allow(non_snake_case)]
pub const fn PLIC_SCLAIM(hart: usize) -> usize { PLIC_SCLAIM_BASE + hart * 0x2000 }

pub const UART0_IRQ: u32 = 10;

pub const VIRTIO0_IRQ: u32 = 1;

pub struct Plic {}

impl Plic {
    pub const fn new() -> Self {
        Self {}
    }
    /// Get the next available interrupt. This is the "claim" process.
    /// The plic will automatically sort by priority and hand us the
    /// ID of the interrupt. For example, if the UART is interrupting
    /// and it's next, we will get the value 10.
    pub fn next(&mut self) -> Option<u32> {
        let claim_reg = PLIC_SCLAIM(hart_id()) as *const u32;
        let claim_no;
        // The claim register is filled with the highest-priority, enabled interrupt.
        unsafe {
            claim_no = claim_reg.read_volatile();
        }
        if claim_no == 0 {
            // The interrupt 0 is hardwired to 0, which tells us that there is no
            // interrupt to claim, hence we return None.
            None
        } else {
            // If we get here, we've gotten a non-0 interrupt.
            Some(claim_no)
        }
    }

    /// Complete a pending interrupt by id. The id should come
    /// from the next() function above.
    pub fn complete(&mut self, id: u32) {
        let complete_reg = PLIC_SCLAIM(hart_id()) as *mut u32;
        unsafe {
            // We actually write a u32 into the entire complete_register.
            // This is the same register as the claim register, but it can
            // differentiate based on whether we're reading or writing.
            complete_reg.write_volatile(id);
        }
    }

    /// Initialize PLIC. Enable interrupt.
    pub unsafe fn init(&mut self, id: u32) {
        let enables = PLIC_BASE as *mut u32;
        enables.add(id as usize).write_volatile(1);
    }

    /// See if a given interrupt id is pending.
    ///
    /// Should only be called with lock.
    pub unsafe fn is_pending(&mut self, id: u32) -> bool {
        let pend = PLIC_PENDING as *const u32;
        let actual_id = 1 << id;
        let pend_ids;
        pend_ids = pend.read_volatile();
        actual_id & pend_ids != 0
    }

    /// Enable a given interrupt id
    pub fn enable(&mut self, id: u32) {
        let enables = PLIC_SENABLE(hart_id()) as *mut u32;
        let actual_id = 1 << id;
        unsafe {
            // Unlike the complete and claim registers, the plic_int_enable
            // register is a bitset where the id is the bit index. The register
            // is a 32-bit register, so that gives us enables for interrupts
            // 31 through 1 (0 is hardwired to 0).
            enables.write_volatile(enables.read_volatile() | actual_id);
        }
    }

    /// Set the global threshold. The threshold can be a value [0..7].
    /// The PLIC will mask any interrupts at or below the given threshold.
    /// This means that a threshold of 7 will mask ALL interrupts and
    /// a threshold of 0 will allow ALL interrupts.
    pub fn set_threshold(&mut self, tsh: u8) {
        // We do tsh because we're using a u8, but our maximum number
        // is a 3-bit 0b111. So, we and with 7 (0b111) to just get the
        // last three bits.
        let actual_tsh = tsh & 7;
        let tsh_reg = PLIC_SPRIORITY(hart_id()) as *mut u32;
        unsafe {
            tsh_reg.write_volatile(actual_tsh as u32);
        }
    }

    /// Set a given interrupt priority to the given priority.
    /// The priority must be [0..7]
    pub fn set_priority(&mut self, id: u32, prio: u8) {
        let actual_prio = prio as u32 & 7;
        let prio_reg = PLIC_SPRIORITY(hart_id()) as *mut u32;
        unsafe {
            // The offset for the interrupt id is:
            // PLIC_PRIORITY + 4 * id
            // Since we're using pointer arithmetic on a u32 type,
            // it will automatically multiply the id by 4.
            prio_reg.add(id as usize).write_volatile(actual_prio);
        }
    }
}

/// PLIC driver object
static mut __PLIC: Plic = Plic::new();

/// Global function to get an instance of PLIC driver
#[allow(non_snake_case)]
pub fn PLIC() -> &'static mut Plic { unsafe { &mut __PLIC } }

/// Initialize PLIC
///
/// This function should only be called from boot hart
pub unsafe fn init() {
    let plic = PLIC();
    plic.init(UART0_IRQ);
    plic.init(VIRTIO0_IRQ);
}

pub fn hartinit() {
    let plic = PLIC();
    plic.enable(UART0_IRQ);
    plic.enable(VIRTIO0_IRQ);
    plic.set_threshold(0);
    plic.set_priority(UART0_IRQ, 1);
    plic.set_priority(VIRTIO0_IRQ, 1);
}
