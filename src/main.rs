#![no_std]
#![no_main]

mod uart;

use core::{
    arch::{asm, global_asm},
    fmt::Write,
    panic::PanicInfo,
};

global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let _ = uart::UART.lock().write_str("Hello world!");

    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
