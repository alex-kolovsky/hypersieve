#![no_std]
#![no_main]

mod allocator;
mod arch;
mod guest;
mod handle_trap;
mod mmio;
mod multihart;
mod vcpu_functions;

extern crate alloc;

use crate::arch::extension_routines;
use crate::{
    allocator::BUDDY_ALLOCATOR,
    arch::uart::global_uart_init,
    arch::{HardwareInterface, HartFunctions, HartInterface},
    guest::Guest,
};

// Include files created by the build script.
include!(concat!(env!("OUT_DIR"), "/guest_constants.rs"));
include!(concat!(env!("OUT_DIR"), "/hart_constants.rs"));
include!(concat!(env!("OUT_DIR"), "/guest_specific_routines.rs"));

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
    global_uart_init();

    HartFunctions::hart_configure();

    guest::initialize_guests();

    // Start all other harts if available.
    HartFunctions::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}
