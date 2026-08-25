#![no_std]
#![no_main]

mod allocator;
mod guest;
mod guest_table;
mod multihart;
mod trap;
mod uart;
mod vcpu;

extern crate alloc;

use crate::{allocator::BUDDY_ALLOCATOR, guest::Guest, uart::UART};
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

global_asm!(include_str!("asm/boot.S"));

include!(concat!(env!("OUT_DIR"), "/guests.rs"));

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
pub extern "C" fn main(hart_id: usize) -> ! {
    BUDDY_ALLOCATOR.heap.lock().init();
    UART.lock().init();

    multihart::hart_configure();

    // Fill GUESTS array with VCPUs.
    for guest in GUESTS.iter() {
        let vcpu_ptr: *mut crate::vcpu::Vcpu = guest.vcpu.get();

        unsafe {
            *vcpu_ptr = guest::allocate_guest_memory(guest.entry_gpa, guest.data);
        }

        guest
            .vcpu_ptr
            .store(vcpu_ptr, core::sync::atomic::Ordering::Release);
    }

    // Start all other harts if available.
    multihart::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}
