#![no_std]
#![no_main]

mod allocator;
mod trap;
mod uart;

extern crate alloc;

use crate::allocator::BUDDY_ALLOCATOR;
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
    {
        let mut heap = BUDDY_ALLOCATOR.heap.lock();
        heap.init();
    }

    println!("Hello world!");

    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
