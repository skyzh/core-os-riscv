// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Never type and jump after RAII

fn do_nothing() {}

/// A mock function with return type !
///
/// This can be further optimized
#[inline(always)]
fn never_return() -> ! {
    let func: extern "C" fn() -> ! = unsafe { core::mem::transmute(do_nothing as *const ()) };
    func()
}

/// Return to a non-return function
#[inline(always)]
pub fn return_to(func: fn() -> !) -> ! {
    let fn_addr = func as *const () as usize;
    unsafe { crate::arch::w_ra(fn_addr); }
    never_return()
}
