
#[macro_export]
macro_rules! symbol {
    ( $x:ident, $y:ident, $type:ty ) => {
        extern "C" { static $x: $type; }
        pub const $y: $type = unsafe { $x };
    }
}

extern "C" { static __heap_start: usize; }
pub fn HEAP_START() -> usize { unsafe { &__heap_start as *const _ as _ } }
extern "C" { static __heap_size: usize; }
pub fn HEAP_SIZE() -> usize { unsafe { &__heap_size as *const _ as _ } }
extern "C" { static __text_start: usize; }
pub fn TEXT_START() -> usize { unsafe { &__text_start as *const _ as _ } }
extern "C" { static __text_end: usize; }
pub fn TEXT_END() -> usize { unsafe { &__text_end as *const _ as _ } }
extern "C" { static __rodata_start: usize; }
pub fn RODATA_START() -> usize { unsafe { &__rodata_start as *const _ as _ } }
extern "C" { static __rodata_end: usize; }
pub fn RODATA_END() -> usize { unsafe { &__rodata_end as *const _ as _ } }
extern "C" { static __data_start: usize; }
pub fn DATA_START() -> usize { unsafe { &__data_start as *const _ as _ } }
extern "C" { static __data_end: usize; }
pub fn DATA_END() -> usize { unsafe { &__data_end as *const _ as _ } }
extern "C" { static __bss_start: usize; }
pub fn BSS_START() -> usize { unsafe { &__bss_start as *const _ as _ } }
extern "C" { static __bss_end: usize; }
pub fn BSS_END() -> usize { unsafe { &__bss_end as *const _ as _ } }
extern "C" { static __kernel_stack_start: usize; }
pub fn KERNEL_STACK_START() -> usize { unsafe { &__kernel_stack_start as *const _ as _ } }
extern "C" { static __kernel_stack_end: usize; }
pub fn KERNEL_STACK_END() -> usize { unsafe { &__kernel_stack_end as *const _ as _ } }
extern "C" { static __trampoline_text_start: usize; }
pub fn TRAMPOLINE_TEXT_START() -> usize { unsafe { &__trampoline_text_start as *const _ as _ } }
