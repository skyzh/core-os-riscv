// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! All tests

use crate::virtio;
use crate::info;

pub fn run_tests() {
    let suites = [("virtio", crate::virtio::tests::tests)];
    for (name, suite) in &suites {
        let tests = suite();
        info!("  {}", name);
        for (name, func) in tests {
            func();
            info!("    {}... \x1b[0;32mok\x1b[0m", name);
        }
    }
}
