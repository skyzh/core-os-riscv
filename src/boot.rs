global_asm!(include_str!("boot.S"));

use riscv::{asm, register::*};

#[no_mangle]
pub unsafe extern "C" fn start() -> ! {
    if mhartid::read() == 0 {
        crate::runtime_init::runtime_init();
    }
    loop {
        asm::wfi();
    }
}

