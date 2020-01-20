// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use core::cell::UnsafeCell;
use core::marker::Sync;
use core::ops::{Drop, Deref, DerefMut};
use core::fmt;
use core::option::Option::{self, None, Some};
use core::default::Default;
use crate::{panic, println, panic_println};

/// This type provides MUTual EXclusion based on spinning.
///
/// # Description
///
/// The behaviour of these lock is similar to their namesakes in `std::sync`. they
/// differ on the following:
///
/// - The lock will not be poisoned in case of failure;
///
/// # Simple examples
///
/// ```
/// use spin;
/// let spin_mutex = spin::Mutex::new(0);
///
/// // Modify the data
/// {
///     let mut data = spin_mutex.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = spin_mutex.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
///
/// # Thread-safety example
///
/// ```
/// use spin;
/// use std::sync::{Arc, Barrier};
///
/// let numthreads = 1000;
/// let spin_mutex = Arc::new(spin::Mutex::new(0));
///
/// // We use a barrier to ensure the readout happens after all writing
/// let barrier = Arc::new(Barrier::new(numthreads + 1));
///
/// for _ in (0..numthreads)
/// {
///     let my_barrier = barrier.clone();
///     let my_lock = spin_mutex.clone();
///     std::thread::spawn(move||
///     {
///         let mut guard = my_lock.lock();
///         *guard += 1;
///
///         // Release the lock to prevent a deadlock
///         drop(guard);
///         my_barrier.wait();
///     });
/// }
///
/// barrier.wait();
///
/// let answer = { *spin_mutex.lock() };
/// assert_eq!(answer, numthreads);
/// ```
pub struct Mutex<T: ?Sized>
{
    name: &'static str,
    lock: UnsafeCell<bool>,
    data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
#[derive(Debug)]
pub struct MutexGuard<'a, T: ?Sized + 'a>
{
    name: &'static str,
    lock: &'a UnsafeCell<bool>,
    data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T>
{
    /// Creates a new spinlock wrapping the supplied data.
    ///
    /// May be used statically:
    ///
    /// ```
    /// use spin;
    ///
    /// static MUTEX: spin::Mutex<()> = spin::Mutex::new(());
    ///
    /// fn demo() {
    ///     let lock = MUTEX.lock();
    ///     // do something with lock
    ///     drop(lock);
    /// }
    /// ```
    pub const fn new(user_data: T, name: &'static str) -> Mutex<T>
    {
        Mutex
            {
                lock: UnsafeCell::new(false),
                data: UnsafeCell::new(user_data),
                name
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

impl<T: ?Sized> Mutex<T>
{
    fn obtain_lock(&self)
    {
        /*
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false
        {
            // Wait until the lock looks unlocked before retrying
            while self.lock.load(Ordering::Relaxed)
            {
                cpu_relax();
            }
        }
        */
        unsafe {
            if *self.lock.get() == true {
                panic!("{} already locked!", self.name);
            }
            *self.lock.get() = true;
        }
    }

    /// Locks the spinlock and returns a guard.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    ///
    /// ```
    /// let mylock = spin::Mutex::new(0);
    /// {
    ///     let mut data = mylock.lock();
    ///     // The lock is now locked and the data can be accessed
    ///     *data += 1;
    ///     // The lock is implicitly dropped
    /// }
    ///
    /// ```
    pub fn lock(&self) -> MutexGuard<T>
    {
        self.obtain_lock();
        // panic_println!("{} locked", self.name);
        MutexGuard {
            name: self.name,
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Force unlock the spinlock.
    ///
    /// This is *extremely* unsafe if the lock is not held by the current
    /// thread. However, this can be useful in some instances for exposing the
    /// lock to FFI that doesn't know how to deal with RAII.
    ///
    /// If the lock isn't held, this is a no-op.
    pub unsafe fn force_unlock(&self) {
        // self.lock.store(false, Ordering::Release);
    }

    /// Tries to lock the mutex. If it is already locked, it will return None. Otherwise it returns
    /// a guard within Some.
    pub fn try_lock(&self) -> Option<MutexGuard<T>>
    {
        if unsafe { *self.lock.get() } == false {
            Some(
                MutexGuard {
                    name: self.name,
                    lock: &self.lock,
                    data: unsafe { &mut *self.data.get() },
                }
            )
        } else {
            None
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.try_lock()
            {
                Some(guard) => write!(f, "Mutex {{ data: ")
                    .and_then(|()| (&*guard).fmt(f))
                    .and_then(|()| write!(f, "}}")),
                None => write!(f, "Mutex {{ <locked> }}"),
            }
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default(), "default")
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T>
{
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        unsafe {
            if !*self.lock.get() {
                panic!("{} not locked!", self.name);
            }
            // panic_println!("{} dropped", self.name);
            *self.lock.get() = false;
        }
        // self.lock.store(false, Ordering::Release);
    }
}
