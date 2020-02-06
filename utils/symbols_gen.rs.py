#!/usr/bin/env python3

### Copyright (c) 2020 Alex Chi
### 
### This software is released under the MIT License.
### https://opensource.org/licenses/MIT

"""
#[macro_export]
macro_rules! symbol {
    ( $x:ident, $y:ident, $type:ty ) => {
        extern "C" { static $x: $type; }
        pub const $y: $type = unsafe { $x };
    }
}
"""

print("""//! This module is automatically generated with `symbols_gen.rs.py`,
//! which contains all linker script symbols in `kernel.ld` and a wrapper function
//! to safely get them.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
""")

from symbols import symbols

for symbol in symbols:
    # print("symbol! { __%s, %s, usize }" % (symbol, symbol))
    print("extern \"C\" { static __%s: usize; }" % symbol.lower())
    print("#[inline] pub fn %s() -> usize { unsafe { &__%s as *const _ as _ } }" % (symbol, symbol.lower()))
    