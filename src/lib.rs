#![no_std]

pub(crate) mod allocator;
pub(crate) mod arch;
pub(crate) mod guest;
pub(crate) mod handle_trap;
pub(crate) mod multihart;
pub(crate) mod vcpu_functions;

extern crate alloc;

use crate::{
    arch::{HardwareInterface, HartFunctions, HartInterface, extension_routines},
    guest::Guest,
};

// Include files created by the build script.
include!(concat!(env!("OUT_DIR"), "/guest_constants.rs"));
include!(concat!(env!("OUT_DIR"), "/hart_constants.rs"));
include!(concat!(env!("OUT_DIR"), "/guest_specific_routines.rs"));

pub fn run(hart_id: usize) -> ! {
    arch::uart::global_uart_init();
    allocator::init_heap();

    guest::initialize_guests();

    // Start all other harts if available.
    HartFunctions::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}

#[doc(hidden)]
pub fn print_and_wfi(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {
        arch::Cpu::wait_for_interrupt();
    }
}
