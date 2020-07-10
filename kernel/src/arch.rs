// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V related functions

use core::time::Duration;
use crate::panic;
use riscv::register::*;
use crate::symbols::*;
use core::sync::atomic::Ordering;

/// Get current time from MMIO
pub fn time() -> Duration {
    let mtime = crate::clint::CLINT_MTIME_BASE as *const u64;
    Duration::from_nanos(unsafe { mtime.read_volatile() } * 100)
}

/// Build satp value from mode, asid and page table base addr
pub fn build_satp(mode: usize, asid: usize, addr: usize) -> usize {
    if addr % PAGE_SIZE != 0 {
        panic!("satp not aligned!");
    }
    (mode as usize) << 60 | (asid & 0xffff) << 44 | (addr >> 12) & 0xff_ffff_ffff
}

/// Enable interrupt
pub fn intr_on() {
    use crate::panic_println;
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
        sstatus::set_sie();
    }
}

/// Turn off interrupt
pub fn intr_off() {
    use crate::panic_println;
    unsafe {
        sstatus::clear_sie();
    }
}

/// Check if interrupt is enabled
pub fn intr_get() -> bool {
    sstatus::read().sie()
}

#[inline(always)]
#[allow(unused_assignments)]
pub fn hart_id() -> usize {
    let mut hart_id: usize = 0;
    unsafe { llvm_asm!("mv $0, tp" : "=r"(hart_id) ::: "volatile"); }
    hart_id
}

#[inline]
#[allow(unused_assignments)]
pub fn r_sip() -> usize {
    let mut sip: usize = 0;
    unsafe { llvm_asm!("csrr $0, sip" : "=r"(sip) ::: "volatile"); }
    sip
}

#[inline]
pub fn w_sip(x: usize) {
    unsafe { llvm_asm!("csrw sip, $0" :: "r"(x) :: "volatile"); }
}

#[inline]
#[allow(unused_assignments)]
pub fn r_sstatus() -> usize {
    let mut x: usize = 0;
    unsafe { llvm_asm!("csrr $0, sstatus" : "=r"(x) ::: "volatile"); }
    x
}

#[inline]
#[allow(unused_assignments)]
pub fn r_satp() -> usize {
    let mut x: usize = 0;
    unsafe { llvm_asm!("csrr $0, satp" : "=r"(x) ::: "volatile"); }
    x
}

#[inline]
pub fn w_sstatus(x: usize) {
    unsafe { llvm_asm!("csrw sstatus, $0" :: "r"(x) :: "volatile"); }
}

#[inline(always)]
pub fn __sync_synchronize() {
    core::sync::atomic::compiler_fence(Ordering::SeqCst);
    unsafe { asm!("fence"); }
}

#[inline(always)]
pub fn __sync_lock_test_and_set(a: &u32, mut b: u32) -> u32 {
    unsafe { llvm_asm!("amoswap.w.aq $0, $1, ($2)" :"=r"(b): "r"(b), "r"(a) :: "volatile"); }
    b
}

#[inline(always)]
pub fn __sync_lock_release(a: &u32) {
    unsafe { llvm_asm!("amoswap.w zero, zero, ($0)" :: "r"(a) :: "volatile"); }
}

#[inline(always)]
pub unsafe fn w_ra(x: usize) {
    llvm_asm!("mv ra, $0" :: "r"(x) :: "volatile");
}


pub fn sp() -> usize {
    let mut sp: usize = 0;
    unsafe { llvm_asm!("mv $0, sp" : "=r"(sp) ::: "volatile"); }
    sp
}

pub fn wait_forever() -> ! {
    loop {
        unsafe {
            riscv::asm::wfi();
        }
    }
}
