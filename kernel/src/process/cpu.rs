// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::{Process, TrapFrame, Context};
use alloc::boxed::Box;
use core::mem::MaybeUninit;

/// Holding CPU information
#[repr(C)]
pub struct CPU {
    pub scheduler_context: Context,
    pub process: Option<Box<Process>>,
    pub intr_locker: IntrLocker
}

impl CPU {
    pub const fn zero() -> Self {
        Self {
            process: None,
            scheduler_context: Context::zero(),
            intr_locker: IntrLocker::new()
        }
    }
}

/// Control interrupt
pub struct IntrLocker {
    pub is_locked_before: bool,
    pub cnt: usize
}

impl IntrLocker {
    pub const fn new() -> Self {
        Self {
            is_locked_before: false,
            cnt: 0
        }
    }
}
