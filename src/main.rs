#![no_std]
#![no_main]

mod allocator;
mod guest_table;
mod trap;
mod uart;
mod vcpu;

extern crate alloc;

use crate::{
    allocator::{BUDDY_ALLOCATOR, alloc_pages},
    guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X},
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
pub extern "C" fn main() -> ! {
    {
        let mut heap = BUDDY_ALLOCATOR.heap.lock();
        heap.init();
    }

    // Set the VS Timer Interrupt Enable (VSTIE) bit in hie to allow timer interrupts in VS mode.
    let hie_vstie = 1 << 6;
    let hie = read_csr!("hie") | hie_vstie; // hie: Hypervisor Interrupt Enable

    // Set the Supervisor Timer Interrupt Enable (STIE) bit in vsie to handle guest timer interrupts.
    let vsie_stie = 1 << 6;
    let vsie = read_csr!("vsie") | vsie_stie; // vsie: Virtual Supervisor Interrupt Enable

    // Set the STimecmp Enable (STCE) bit in henvcfg to enable S/VS mode time comparators.
    let henvcfg_stce = 1 << 63;
    let henvcfg = read_csr!("henvcfg") | henvcfg_stce; // henvcfg: Hypervisor Environment Configuration

    unsafe {
        asm!(
            "csrw hie, {hie}",
            "csrw henvcfg, {henvcfg}",
            "csrw vsie, {vsie}",
            hie = in(reg) hie,
            henvcfg = in(reg) henvcfg,
            vsie = in(reg) vsie,
        );
    }

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
