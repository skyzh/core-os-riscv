// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

/// Get file in the fake filesystem. Maximum file size is 1048576 (1M).
pub fn get_file(filename: &str) -> (*const u8, usize) {
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
        "/init" => (init.as_ptr(), init.len()),
        "/test1" => (test1.as_ptr(), test1.len()),
        "/test2" => (test2.as_ptr(), test2.len()),
        "/test3" => (test3.as_ptr(), test3.len()),
        _ => unreachable!()
    }
}
