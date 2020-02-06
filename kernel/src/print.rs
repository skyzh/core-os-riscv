// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Macros for output

use core::fmt;
use crate::spinlock::Mutex;

pub static INFO_LOCK: Mutex<()> = Mutex::new((), "info");

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	let mut uart = crate::uart::UART().lock();
	uart.write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _panic_print(args: fmt::Arguments) {
    use core::fmt::Write;
    use crate::uart::*;
	let mut uart = Uart::new(UART_BASE_ADDR);
	uart.write_fmt(args).unwrap();
}

/// Print information
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Print with a new line
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

/// Create a new UART object and print
#[macro_export]
macro_rules! panic_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_panic_print(format_args_nl!($($arg)*));
    })
}

/// Prints an info, with newline.
#[macro_export]
macro_rules! info {
    ($string:expr) => ({
        #[allow(unused_imports)]
        let _info_locker = $crate::print::INFO_LOCK.lock();

        let timestamp = $crate::arch::time();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("\x1b[0;36m[  {:>3}.{:03}{:03}]\x1b[0m ", $string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        #[allow(unused_imports)]
        let _info_locker = $crate::print::INFO_LOCK.lock();

        let timestamp = $crate::arch::time();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("\x1b[0;36m[  {:>3}.{:03}{:03}]\x1b[0m ", $format_string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000,
            $($arg)*
        ));
    })
}

/// Prints a warning, with newline.
#[macro_export]
macro_rules! warn {
    ($string:expr) => ({
        #[allow(unused_imports)]
        let _info_locker = $crate::print::INFO_LOCK.lock();

        let timestamp = $crate::arch::time();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("[W {:>3}.{:03}{:03}] ", $string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        #[allow(unused_imports)]
        let _info_locker = $crate::print::INFO_LOCK.lock();

        let timestamp = $crate::arch::time();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("[W {:>3}.{:03}{:03}] ", $format_string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000,
            $($arg)*
        ));
    })
}
