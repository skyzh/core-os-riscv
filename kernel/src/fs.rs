// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

/// Get file in the fake filesystem. Maximum file size is 1048576 (1M).
pub fn get_file(filename: &str) -> &'static [u8] {
    let init;
    let test1;
    let test2;
    let test3;
    #[cfg(debug_assertions)] {
        test1 = include_bytes!("../../target/riscv64gc-unknown-none-elf/debug/test1");
        test2 = include_bytes!("../../target/riscv64gc-unknown-none-elf/debug/test2");
        test3 = include_bytes!("../../target/riscv64gc-unknown-none-elf/debug/test3");
        init = include_bytes!("../../target/riscv64gc-unknown-none-elf/debug/init");
    }
    #[cfg(not(debug_assertions))] {
        test1 = include_bytes!("../../target/riscv64gc-unknown-none-elf/release/test1");
        test2 = include_bytes!("../../target/riscv64gc-unknown-none-elf/release/test2");
        test3 = include_bytes!("../../target/riscv64gc-unknown-none-elf/release/test3");
        init = include_bytes!("../../target/riscv64gc-unknown-none-elf/release/init");
    }
    match filename {
        "/init" => init,
        "/test1" => test1,
        "/test2" => test2,
        "/test3" => test3,
        _ => unreachable!()
    }
}
