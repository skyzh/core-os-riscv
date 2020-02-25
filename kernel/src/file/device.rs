// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Device trait for devices such as Console

use crate::uart::UART;

/// Device trait
///
/// All device should implement their own synchronize mechanisms.
pub trait Device: Send + Sync {
    /// Read from file to content and returns number of characters (<= `content.len()`) read.
    fn read(&self, content: &mut [u8]) -> i32;
    /// Write content to file and returns number of characters written.
    fn write(&self, content: &[u8]) -> i32;
}

/// Console device
pub struct Console {}

impl Device for Console {
    /// read from console
    fn read(&self, content: &mut [u8]) -> i32 {
        let mut uart = UART().lock();
        for i in 0..content.len() {
            match uart.get() {
                Some(ch) => { content[i] = ch; }
                _ => { return i as i32; }
            }
        }
        return content.len() as i32;
    }

    /// write to console
    fn write(&self, content: &[u8]) -> i32 {
        let mut uart = UART().lock();
        for i in 0..content.len() {
            uart.put(content[i]);
        }
        return content.len() as i32;
    }
}
