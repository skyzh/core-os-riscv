// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::{Process, TrapFrame, Context};
use alloc::boxed::Box;
use core::mem::MaybeUninit;

#[repr(C)]
pub struct CPU {
    pub kernel_trapframe: TrapFrame,
    pub scheduler_context: Context,
    pub process: Option<Box<Process>>
}

impl CPU {
    pub const fn zero() -> Self {
        Self {
            process: None,
            kernel_trapframe: TrapFrame::zero(),
            scheduler_context: Context::zero()
        }
    }
}
