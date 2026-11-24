#![no_std]

pub(crate) mod allocator;
pub(crate) mod arch;
pub(crate) mod guest;
pub(crate) mod handle_trap;
pub(crate) mod multihart;
pub(crate) mod vcpu_functions;
pub(crate) mod wasm;

extern crate alloc;

// Include files created by the build script.
include!(concat!(codegen_out_dir!(), "/guest_constants.rs"));
include!(concat!(codegen_out_dir!(), "/hart_constants.rs"));
include!(concat!(codegen_out_dir!(), "/guest_specific_routines.rs"));
include!(concat!(codegen_out_dir!(), "/module_mappings.rs"));

use crate::{
    arch::{HardwareInterface, HartFunctions, HartInterface, extension_routines},
    guest::Guest,
};
use hypersieve_codegen::codegen_out_dir;

pub fn run(hart_id: usize) -> ! {
    arch::uart::global_uart_init();
    allocator::init_heap();

    HartFunctions::hart_configure();

    guest::initialize_guests();

    wasm::init_engine();
    wasm::init_wasm_modules();

    // Start all other harts if available.
    HartFunctions::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}

#[doc(hidden)]
pub fn print_and_wfi(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {
        arch::Cpu::wait_for_interrupt();
    }
}
