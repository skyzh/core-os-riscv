
#[macro_export]
macro_rules! symbol {
    ( $x:ident, $y:ident, $type:ty ) => {
        extern "C" { static $x: $type; }
        pub const $y: $type = unsafe { $x };
    }
}

extern "C" { pub static HEAP_START: usize; }
extern "C" { pub static HEAP_SIZE: usize; }
extern "C" { pub static TEXT_START: usize; }
extern "C" { pub static TEXT_END: usize; }
extern "C" { pub static RODATA_START: usize; }
extern "C" { pub static RODATA_END: usize; }
extern "C" { pub static DATA_START: usize; }
extern "C" { pub static DATA_END: usize; }
extern "C" { pub static BSS_START: usize; }
extern "C" { pub static BSS_END: usize; }
extern "C" { pub static KERNEL_STACK_START: usize; }
extern "C" { pub static KERNEL_STACK_END: usize; }
extern "C" { pub static TRAMPOLINE_TEXT_START: usize; }
