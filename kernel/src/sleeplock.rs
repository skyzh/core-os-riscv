// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! A lock which puts current process into sleep and allows interrupt

use crate::spinlock::{Mutex, MutexGuard};
use crate::process::{sleep, my_proc, wakeup};
use crate::info;

/// locked, pid
struct SleepLockInfo {
    pub locked: bool,
    pub pid: i32,
}

impl SleepLockInfo {
    pub const fn new(locked: bool, pid: i32) -> Self {
        Self { locked, pid }
    }
}

pub struct SleepLock {
    spin: Mutex<SleepLockInfo>,
    name: &'static str,
}

pub struct SleepLockGuard<'a> {
    lock: &'a SleepLock,
}

impl SleepLock {
    pub const fn new(name: &'static str) -> Self {
        Self {
            spin: Mutex::new(SleepLockInfo::new(false, 0), "sleep lock"),
            name,
        }
    }

    pub fn acquire(&mut self) -> SleepLockGuard {
        let mut lk = self.spin.lock();
        while lk.locked {
            lk = sleep(&self, lk);
        }
        lk.locked = true;
        lk.pid = my_proc().pid;
        SleepLockGuard {
            lock: self
        }
    }

    pub fn holding(&self) -> bool {
        let lk = self.spin.lock();
        lk.locked && lk.pid == my_proc().pid
    }
}

impl Drop for SleepLockGuard<'_> {
    fn drop(&mut self) {
        let mut lk = self.lock.spin.lock();
        lk.locked = false;
        lk.pid = 0;
        wakeup(&self.lock);
    }
}
