// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

// core-os-riscv spin-lock is based on Rust crate `spin`.

//The MIT License (MIT)
//
//Copyright (c) 2014 Mathijs van de Nes
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

//! A RISC-V Mutex

use core::cell::UnsafeCell;
use core::marker::Sync;
use core::ops::{Drop, Deref, DerefMut};
use core::fmt;
use core::option::Option::{self, None, Some};
use core::default::Default;
use crate::{arch, panic, process::IntrLockGuard};
use crate::process::my_cpu;
use crate::panic_println;
use crate::arch::hart_id;
use alloc::rc::Weak;

/// A RISC-V Mutex.
pub struct Mutex<T: ?Sized> {
    /// Indicate whether data is locked. Will be passed into C code.
    lock: u32,
    /// Lock name, for debug use
    name: &'static str,
    /// HartID holding the lock
    hart: UnsafeCell<i64>,
    /// Save actual data
    data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a u32,
    mutex: &'a Mutex<T>,
    data: &'a mut T,
    intr_lock: IntrLockGuard<'a>,
}

pub struct WeakMutexGuard<'a, T: ?Sized + 'a> {
    mutex: &'a Mutex<T>
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(user_data: T, name: &'static str) -> Mutex<T> {
        Mutex {
            lock: 0,
            data: UnsafeCell::new(user_data),
            hart: UnsafeCell::new(-1),
            name,
        }
    }

    /// Consumes this mutex, returning the underlying data.
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let Mutex { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Obtain lock by test_and_set
    fn obtain_lock(&self) {
        while arch::__sync_lock_test_and_set(&self.lock, 1) != 0 {}
        arch::__sync_synchronize();
    }

    /// Lock mutex and return a guard
    pub fn lock(&self) -> MutexGuard<T> {
        let intr_lock = my_cpu().intr_lock.lock();
        if unsafe { self.holding() } {
            panic!("lock {}: hart {} already holding the lock!", self.name, arch::hart_id());
        }
        self.obtain_lock();
        if self.lock != 1 {
            panic!("lock {}: not locked!", self.name);
        }
        unsafe { *self.hart.get() = arch::hart_id() as i64; }
        MutexGuard {
            lock: &self.lock,
            mutex: self,
            data: unsafe { &mut *self.data.get() },
            intr_lock,
        }
    }

    /// Test if lock is held by current hart
    unsafe fn holding(&self) -> bool {
        let _intr_lock = my_cpu().intr_lock.lock();
        if self.lock == 1 && *self.hart.get() == arch::hart_id() as i64 {
            return true;
        }
        false
    }

    /// Directly get mutex data regardless whether it is locked or not
    pub unsafe fn get(&self) -> &mut T {
        &mut *self.data.get()
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self) {
        if unsafe { !self.mutex.holding() } {
            panic!("lock {}: unlock from another hart {}!", self.mutex.name, arch::hart_id());
        }
        unsafe { *self.mutex.hart.get() = -1; }
        arch::__sync_synchronize();
        arch::__sync_lock_release(&self.lock);
        // panic_println!("{} unlock on {}", self.mutex.name, arch::hart_id());
    }
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    /// Temporarily unlock Mutex by obtaining a weak guard
    pub fn into_weak(self) -> WeakMutexGuard<'a, T> {
        WeakMutexGuard {
            mutex: self.mutex
        }
    }
}

impl<'a, T: ?Sized> WeakMutexGuard<'a, T> {
    /// Temporarily unlock Mutex by obtaining a weak guard
    pub fn into_guard(self) -> MutexGuard<'a, T> {
        self.mutex.lock()
    }
}
