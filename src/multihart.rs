use crate::{
    GUESTS, HARTS, MAX_SUPPORTED_ASSIGNED_GUESTS_PER_HART,
    arch::{HartFunctions, HartInterface, VcpuInterface},
};
use core::cell::UnsafeCell;

#[derive(Debug)]
pub struct Hart {
    pub is_assigned: bool,
    pub assigned_guest_vcpus: UnsafeCell<
        [Option<&'static mut dyn VcpuInterface>; MAX_SUPPORTED_ASSIGNED_GUESTS_PER_HART],
    >,
    pub guests: [Option<usize>; MAX_SUPPORTED_ASSIGNED_GUESTS_PER_HART],
}

// SAFETY: The Hart struct is thread-safe because we never change
// assigned_guest_vcpus field after waking the harts up.
unsafe impl Sync for Hart {}
unsafe impl Send for Hart {}

#[unsafe(no_mangle)]
pub fn hart_init(hart_id: usize) -> ! {
    HartFunctions::hart_configure();
    jump_in_guest(hart_id);
}

pub fn jump_in_guest(hart_id: usize) -> ! {
    if HARTS.len() <= hart_id {
        panic!("Excess hart, hart id: {hart_id}");
    }
    let hart = &crate::HARTS[hart_id];
    let is_assigned = hart.is_assigned;
    if is_assigned {
        // Run a guest on an assigned hart.

        // Extract the guests for which this hart is assigned.
        let guests = hart.guests;

        for (local_guest_id, global_guest_id) in guests.iter().enumerate() {
            if let Some(global_guest_id) = global_guest_id {
                // Check if a guest is free. A guest is free if the assigned hart count
                // is less than the active assigned hart count.
                let is_claimed =
                    crate::guest::claim_assigned_hart_slot_if_available(*global_guest_id);

                // SAFETY: This dereference is safe because we have verified that
                // the pointer is valid, and the vcpu_ptrs array, from which
                // we obtained this pointer, was fully initialized before this moment.
                unsafe {
                    // Run a guest if it is free.
                    if is_claimed {
                        let vcpu_ptr = (*hart.assigned_guest_vcpus.get())[local_guest_id]
                            .take()
                            .unwrap();
                        vcpu_ptr.very_fisrt_run(hart_id, local_guest_id);
                    }
                }
            } else {
                break;
            }
        }
    } else {
        // Run a guest on a non-assigned hart.
        for global_guest_id in 0..GUESTS.len() {
            // Check if a guest is free.
            let is_free = crate::guest::claim_vcpu_for_hart_if_available(global_guest_id);
            // Run a guest if it is free; otherwise, continue searching for a free guest.
            if let Some(vcpu_ptr) = is_free {
                // Fill the guest ID field with the guest's position in the GUESTS static array.
                vcpu_ptr.very_fisrt_run(hart_id, global_guest_id);
            }
        }
    }

    // A hart is excess if there are no free guests for it.
    panic!("Couldn't find a job for a hart, hart id: {hart_id}");
}
