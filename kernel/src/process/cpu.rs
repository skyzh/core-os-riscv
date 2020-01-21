// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use super::{Process, TrapFrame, Context};

#[repr(C)]
pub struct CPU {
    pub kernel_trapframe: TrapFrame,
    pub scheduler_context: Context,
    pub process_id: i64
}

impl CPU {
    pub const fn zero() -> Self {
        Self {
            process_id: -1,
            kernel_trapframe: TrapFrame::zero(),
            scheduler_context: Context::zero()
        }
    }
}
