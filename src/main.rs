#![no_std]
#![no_main]

mod trap;
mod uart;

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    println!("Hello world!");

    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
