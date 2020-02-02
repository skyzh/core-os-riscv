#!/usr/bin/env python3

### Copyright (c) 2020 Alex Chi
### 
### This software is released under the MIT License.
### https://opensource.org/licenses/MIT

from syscall import syscalls

print("""//! This module is automatically generated with `syscall_gen.rs.py`,
//! which contains all syscall ID.
""")

# print("#[repr(i64)]")
# print("pub enum Syscall {")
for (idx, syscall) in enumerate(syscalls):
    # print(f"    {syscall.upper()} = {idx},")
    print(f"/// `{idx}`: {syscall}")
    print(f"pub const SYS_{syscall.upper()} : i64 = {idx};")
# print("}")
