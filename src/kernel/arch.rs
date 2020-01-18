// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use core::time::Duration;

pub fn time() -> Duration {
    let mtime = 0x0200_bff8 as *const u64;
    Duration::from_nanos(unsafe { mtime.read_volatile() } * 100)
}
