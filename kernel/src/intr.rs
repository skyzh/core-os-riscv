// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Handle interrupts

use riscv::register::*;
use crate::plic;
use crate::uart::uartintr;
use crate::arch;
use crate::virtio::virtiointr;
use crate::println;

#[derive(PartialEq)]
pub enum Intr {
    Timer,
    Device
}

/// Process device interrupts
pub fn devintr() -> Option<Intr> {
    let cause = scause::read();

    if cause.is_interrupt() && cause.code() == 9 {
        let plic = crate::plic::PLIC();
        if let Some(interrupt) = plic.next() {
            match interrupt {
                plic::UART0_IRQ => {
                    uartintr();
                },
                plic::VIRTIO0_IRQ => {
                    virtiointr();
                },
                _ => {
                    println!("Unrecognized external interrupt: {}", interrupt);
                }
            }
            plic.complete(interrupt);
        }
        Some(Intr::Device)
    } else if cause.is_interrupt() && cause.code() == 1 {
        arch::w_sip(arch::r_sip() & !2);
        Some(Intr::Timer)
    } else {
        None
    }
}
