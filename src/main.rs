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

use crate::{
    allocator::{BUDDY_ALLOCATOR, alloc_pages},
    guest::Guest,
    guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X},
    uart::UART,
};
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
    {
        let mut heap = BUDDY_ALLOCATOR.heap.lock();
        let uart = UART.lock();
        uart.init();
        heap.init();
    }

    multihart::hart_configure();

    // Start all other harts if available.
    multihart::start_harts(hart_id);

    // Fill GUESTS array with VCPUs.
    let mut guests = GUESTS.lock();
    for guest in guests.iter_mut() {
        guest.vcpu = allocate_guest_memory(guest.entry_gpa, guest.data);
    }

    guests[1].vcpu.run();
}

fn allocate_guest_memory(guest_entry_gpa: usize, image: &'static [u8]) -> crate::vcpu::Vcpu {
    // Copy guest kernel to a guest memory buffer.
    let kernel_memory = alloc_pages(image.len());
    unsafe {
        let dst = kernel_memory;
        let src = image.as_ptr();
        core::ptr::copy_nonoverlapping(src, dst, image.len());
    }

    // Map the guest memory into the guest page table.
    let table = GuestPageTable::new();
    table.map(
        guest_entry_gpa as u64,
        kernel_memory as u64,
        PTE_R | PTE_W | PTE_X,
    );

    crate::vcpu::Vcpu::new(&table, guest_entry_gpa as u64)
}
