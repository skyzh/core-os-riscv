#!/usr/bin/env python3

### Copyright (c) 2020 Alex Chi
### 
### This software is released under the MIT License.
### https://opensource.org/licenses/MIT

print("""
#[macro_export]
macro_rules! symbol {
    ( $x:ident, $y:ident, $type:ty ) => {
        extern "C" { static $x: $type; }
        pub const $y: $type = unsafe { $x };
    }
}
""")

from symbols import symbols

for symbol in symbols:
    # print("symbol! { __%s, %s, usize }" % (symbol, symbol))
    print("extern \"C\" { pub static %s: usize; }" % symbol)
