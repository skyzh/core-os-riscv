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
pub struct IntrLock {
    pub is_enabled_before: bool,
    pub cnt: UnsafeCell<usize>,
}

impl IntrLock {
    pub const fn new() -> Self {
        Self {
            is_enabled_before: false,
            cnt: UnsafeCell::new(0),
        }
    }

    pub fn lock(&mut self) -> IntrLockGuard {
        let enabled = arch::intr_get();
        arch::intr_off();
        unsafe {
            if *self.cnt.get() == 0 {
                self.is_enabled_before = enabled;
            } else {
                if enabled {
                    panic!("lock held but intr enabled");
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
