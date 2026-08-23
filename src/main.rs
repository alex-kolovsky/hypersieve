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

    // Start all other harts if available.
    let mut error_code: isize = 0;
    let mut value: isize = 0;
    let mut cur_hart_id: usize = 0;

    while error_code == 0 {
        // Skip main hart.
        if cur_hart_id != hart_id {
            // Allocate stack for hart.
            let stack_size: usize = 1_000_000;
            let sp: usize = alloc_pages(stack_size) as usize;
            let sp_end = sp + stack_size;
            (error_code, value) =
                sbi_hart_start(cur_hart_id, hart_init as *const () as usize, sp_end);
        }
        cur_hart_id += 1;
    }

    // Error code will be equal to 3 if there's no core with that number, it means the other cores have started.
    if error_code != -3 {
        panic!("Failed to start harts.\nError code: {error_code}\nValue: {value}");
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

#[inline(always)]
fn sbi_hart_start(hart_id: usize, start_addr: usize, opaque: usize) -> (isize, isize) {
    let error: isize;
    let value: isize;

    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") hart_id,
            in("a1") start_addr,
            in("a2") opaque,
            in("a6") 0,
            in("a7") 0x48534D,
            lateout("a0") error,
            lateout("a1") value,
        );
    }

    (error, value)
}

fn hart_init(_: usize, sp_end: usize) {
    unsafe {
        core::arch::asm!(
            // Load stack pointer.
            "mv sp, {sp}",
            // Load trap handler function address.
            "la t0, trap_handler",
            "csrw stvec, t0",
            sp = in(reg) sp_end,
        );
        println!("Hello From Hart");
    }
    panic!();
}
