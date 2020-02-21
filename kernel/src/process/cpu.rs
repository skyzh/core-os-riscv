// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::{Process, TrapFrame, Context};
use alloc::boxed::Box;
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;
use crate::{arch, panic};
use crate::arch::hart_id;

/// Holding CPU information
#[repr(C)]
pub struct CPU {
    pub scheduler_context: Context,
    pub process: Option<Box<Process>>,
    pub intr_lock: IntrLock,
}

impl CPU {
    pub const fn zero() -> Self {
        Self {
            process: None,
            scheduler_context: Context::zero(),
            intr_lock: IntrLock::new(),
        }
    }
}

/// Control interrupt
///
/// Equivalent to xv-6 `push_off` and `pop_off` functions.
///
/// ## Examples
/// ```
/// {
///     let intr_lock = my_cpu().intr_lock.lock();
///     // From this point, interrupt is disabled on this hart.
///     // And whether interrupt is enabled before is recorded in the lock.
///     let intr_lock2 = my_cpu().intr_lock.lock();
///     // No effect.
/// }
/// // As all `intr_lock`s are dropped, interrupt may recover to previous
/// // status.
/// ```
pub struct IntrLock {
    pub is_enabled_before: bool,
    pub cnt: UnsafeCell<isize>,
    pub hart_id: usize,
}

impl IntrLock {
    pub const fn new() -> Self {
        Self {
            is_enabled_before: false,
            cnt: UnsafeCell::new(0),
            hart_id: 23333,
        }
    }

    pub fn lock(&mut self) -> IntrLockGuard {
        let enabled = arch::intr_get();
        arch::intr_off();
        unsafe {
            if *self.cnt.get() == 0 {
                self.is_enabled_before = enabled;
                self.hart_id = arch::hart_id();
            } else {
                if enabled {
                    panic!("lock held but intr enabled");
                }
                if self.hart_id != arch::hart_id() {
                    panic!("lock on different hart");
                }
            }
            *self.cnt.get() += 1;
            IntrLockGuard { lock: self }
        }
    }
}

/// IntrLock Guard
pub struct IntrLockGuard<'a> {
    lock: &'a IntrLock
}

impl<'a> Drop for IntrLockGuard<'_> {
    fn drop(&mut self) {
        if arch::intr_get() {
            panic!("{} intr enabled", hart_id());
        }
        unsafe {
            if hart_id() != self.lock.hart_id {
                panic!("unlock on different hart");
            }
            let cnt = self.lock.cnt.get();
            *cnt -= 1;
            if *cnt == 0 {
                if self.lock.is_enabled_before {
                    arch::intr_on();
                }
            }
            if *cnt < 0 {
                panic!("cnt < 0");
            }
        }
    }
}

unsafe impl Sync for IntrLock {}

unsafe impl Send for IntrLock {}
