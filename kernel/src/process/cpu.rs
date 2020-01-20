use super::{Process, TrapFrame, Context};

#[repr(C)]
#[derive(Clone, Copy)]
#[repr(align(4096))]
pub struct CPU {
    pub process: Process,
    pub kernel_trapframe: TrapFrame,
    pub scheduler_context: Context
}

impl CPU {
    pub const fn zero() -> Self {
        Self {
            process: Process::zero(),
            kernel_trapframe: TrapFrame::zero(),
            scheduler_context: Context::zero()
        }
    }
}
