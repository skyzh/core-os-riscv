// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use core::fmt::{Write, Error, self};
use crate::syscall;

struct StdIO {}

impl StdIO {
    pub fn new() -> Self {
        StdIO {}
    }
}

impl Write for StdIO {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        syscall::write(1, out.as_bytes());
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    StdIO::new().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => (core::fmt::format(format_args_nl!($($arg)*)))
}