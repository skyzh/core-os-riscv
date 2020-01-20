// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

extern "C" {
    pub fn __write(fd: i32, content: *const u8, size: i32) -> i32;
    pub fn __exit(code: i32) -> !;
    pub fn __fork() -> i32;
}
