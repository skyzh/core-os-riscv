// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use crate::println;

#[repr(C)]
pub struct Context {
    // ra + sp + s0~s11
    pub regs: [usize; 14]
}

impl Context {
    pub const fn zero() -> Self {
        Self {
            regs: [0; 14]
        }
    }
}

pub enum ContextRegisters {
    ra = 0,
    sp = 1
}

extern "C" {
    fn __swtch(current: &mut Context, to: &mut Context);
}

pub fn swtch(mut to: Context) -> Context {
    let mut context = Context::zero();
    unsafe { __swtch(&mut context, &mut to); }
    context
}
