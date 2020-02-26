// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Test runner

use crate::virtio;
use crate::info;

type TestSuite = fn() -> &'static [(&'static str, fn())];

/// Run all tests in core os
pub fn run_tests() {
    let suites = [
        ("virtio", crate::virtio::tests::tests as TestSuite),
        ("fsfile", crate::file::fsfile::tests::tests as TestSuite)];
    for (name, suite) in &suites {
        let tests = suite();
        info!("  {}", name);
        for (name, func) in tests {
            info!("    {}", name);
            func();
        }
    }
    info!("\x1b[0;32mall tests passed!\x1b[0m");
}
