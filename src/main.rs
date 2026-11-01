#![no_std]
#![no_main]

mod allocator;
mod arch;
mod guest;
mod guest_table;
mod mmio;
mod multihart;
mod registers;
mod trap;
mod uart;
mod vcpu;

extern crate alloc;

use crate::{allocator::BUDDY_ALLOCATOR, arch::HardwareInterface, guest::Guest, uart::UART};

#[allow(unused_import_braces)]
use core::arch::naked_asm;
use core::mem::offset_of;

// Include files created by the build script.
include!(concat!(env!("OUT_DIR"), "/guest_constants.rs"));
include!(concat!(env!("OUT_DIR"), "/hart_constants.rs"));
include!(concat!(env!("OUT_DIR"), "/vector_extension_support.rs"));
include!(concat!(env!("OUT_DIR"), "/guest_specific_routines.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/floating_point_extension_support.rs"
));

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {
        crate::arch::Cpu::wait_for_interrupt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(hart_id: usize) -> ! {
    BUDDY_ALLOCATOR.heap.lock().init();
    UART.lock().init();

    multihart::hart_configure();

    guest::initialize_guests();

    // Start all other harts if available.
    multihart::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}
