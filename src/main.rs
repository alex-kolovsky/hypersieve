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

use crate::{allocator::BUDDY_ALLOCATOR, guest::Guest, uart::UART, vcpu::Vcpu};

core::arch::global_asm!(include_str!("asm/boot.S"));

// Include the file created by the build script.
include!(concat!(env!("OUT_DIR"), "/guests.rs"));

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(hart_id: usize) -> ! {
    BUDDY_ALLOCATOR.heap.lock().init();
    UART.lock().init();

    multihart::hart_configure();

    // Fill the guests with VCPUs.
    for guest in GUESTS.iter() {
        let vcpus: *mut [Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST] = guest.vcpus.get();

        for i in 0..(guest.harts_cap + guest.dedicated_harts_cap) {
            // Get the pointer to the first vCPU in guest.vcpus.
            let base_ptr: *mut Option<Vcpu> = vcpus as *mut Option<vcpu::Vcpu>;

            // Get the vcpu pointer by its ID.
            let cur_vcpu: *mut Option<Vcpu> = unsafe { base_ptr.add(i) };

            unsafe {
                if let Some(vcpu_ref) = &mut *cur_vcpu {
                    let vcpu: *mut Vcpu = vcpu_ref as *mut Vcpu;

                    // Load the allocated vcpu instead of an empty slot into guest.vcpus[i].
                    *vcpu = guest::allocate_guest_memory(guest.entry_gpa, guest.data);

                    // Load this vcpu pointer into guest.vcpu_ptrs[i].
                    let mut vcpu_ptr = guest.vcpu_ptrs.lock();
                    (*vcpu_ptr)[i] = Some(vcpu);
                } else {
                    unreachable!();
                }
            }
        }

        // Initialize dedicated Hart pointers before booting the secondary cores.
        for dedicated_hart_id in guest.dedicated_harts.iter().flatten() {
            let dedicated_to = HARTS[*dedicated_hart_id as usize].dedicated_to.get();

            let mut vcpu_ptrs = guest.vcpu_ptrs.lock();
            unsafe {
                for id in 0..(*dedicated_to).len() {
                    if (*dedicated_to)[id].is_null() {
                        // First values of the vcpu_ptrs array are intended for dedicated harts.
                        (*dedicated_to)[id] = (*vcpu_ptrs)[0].unwrap();

                        // Remove the vcpu pointer of a dedicated hart from the vcpu_ptrs array.
                        (*vcpu_ptrs)[0] = None;
                        (*vcpu_ptrs).rotate_left(1);
                    }
                }
            }
        }
    }

    // Start all other harts if available.
    multihart::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}
