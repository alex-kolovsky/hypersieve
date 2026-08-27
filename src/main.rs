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

// Include file created by build scrip.
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

        for i in 0..guest.harts_cap {
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
                    let vcpu_ptr = guest.vcpu_ptrs.get();
                    (*vcpu_ptr)[i] = Some(vcpu);
                } else {
                    unreachable!();
                }
            }
        }

        // Initialize dedicated Hart pointers before booting the secondary cores.
        if let Some(dedicated_harts) = guest.dedicated_harts {
            for (i, hart_id) in dedicated_harts.iter().enumerate() {
                let dedicated_ptr = HARTS[*hart_id as usize].dedicated_to.get();

                unsafe {
                    for dedicated_id in 0..(*dedicated_ptr).len() {
                        if (*dedicated_ptr)[dedicated_id].is_null() {
                            (*dedicated_ptr)[dedicated_id] = (*guest.vcpu_ptrs.get())[i].unwrap();
                        }
                    }
                }
            }
        }
    }

    // Start all other harts if available.
    multihart::start_harts(hart_id);

    multihart::jump_in_guest(hart_id);
}
