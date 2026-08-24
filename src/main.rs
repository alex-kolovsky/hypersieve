#![no_std]
#![no_main]

mod allocator;
mod guest_table;
mod multihart;
mod trap;
mod uart;
mod vcpu;

extern crate alloc;

use crate::{
    allocator::{BUDDY_ALLOCATOR, alloc_pages},
    guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X},
    uart::UART,
    vcpu::Vcpu,
};
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

    // Include guest binary file.
    let kernel_image = include_bytes!("../target/guest/guest.bin");
    let guest_entry = 0x100000;

    // Copy guest kernel to a guest memory buffer.
    let kernel_memory = alloc_pages(kernel_image.len());
    unsafe {
        let dst = kernel_memory;
        let src = kernel_image.as_ptr();
        core::ptr::copy_nonoverlapping(src, dst, kernel_image.len());
    }

    // Map the guest memory into the guest page table.
    let table = GuestPageTable::new();
    table.map(guest_entry, kernel_memory as u64, PTE_R | PTE_W | PTE_X);

    // Switch to VS mode.
    let mut vcpu = Vcpu::new(&table, guest_entry);
    vcpu.run();
}
