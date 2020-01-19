#!/usr/bin/env python3

### Copyright (c) 2020 Alex Chi
### 
### This software is released under the MIT License.
### https://opensource.org/licenses/MIT

from syscall import syscalls

for (idx, syscall) in enumerate(syscalls):
    print(f"""
.global __{syscall}
__{syscall}:
li a7, {idx}
ecall
ret""")
