#!/usr/bin/env python3

### Copyright (c) 2020 Alex Chi
### 
### This software is released under the MIT License.
### https://opensource.org/licenses/MIT

from symbols import symbols

print(".section .rodata")
for symbol in symbols:
    print(f".global {symbol}")
    print(f"{symbol}: .dword __{symbol.lower()}")
