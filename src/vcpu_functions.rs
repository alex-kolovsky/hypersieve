use crate::{GUESTS, arch::VcpuInterface};
use core::sync::atomic::Ordering;

const GUESTS_LEN: usize = GUESTS.len();
const GUEST_INDEXES: [usize; GUESTS_LEN] = {
    let mut guest_indexes = [0; GUESTS_LEN];
    let mut i = 0;
    while i != GUESTS_LEN {
        guest_indexes[i] = i;
        i += 1;
    }
    guest_indexes
};

pub fn run_next_guest(cur_vcpu_ref: &'static mut dyn VcpuInterface) {
    // Save these values before releasing this vcpu
    // to other harts to prevent data races.
    let host_sp = cur_vcpu_ref.host_stack_pointer();
    let host_hart_id = cur_vcpu_ref.host_hart_id();

    let current_guest = cur_vcpu_ref.current_guest_ref();
    let guest_id = current_guest.id;

    let is_assigned = crate::multihart::is_hart_assigned(host_hart_id);

    let next_guest_vcpu: *mut dyn VcpuInterface;

    let guest_id_for_hart = if is_assigned {
        // Decrement the global counter of active harts for this guest,
        // allowing another waiting hart to claim and run it.
        current_guest
            .active_assigned_hart_count
            .fetch_sub(1, Ordering::Acquire);

        let hart = &crate::HARTS[cur_vcpu_ref.host_hart_id()];

        // Rotate the assigned guest IDs array to start from the next sequential guest.
        let mut guest_ids = hart.guests;
        guest_ids.rotate_left(guest_id);

        let local_guest_id;

        // Search for an available guest and claim it for this hart.
        (local_guest_id, next_guest_vcpu) = claim_assigned_guest(&guest_ids, hart);
        local_guest_id
    } else {
        // If a hart is not assigned, release this vcpu from the global Guest struct.
        let mut vcpu_ptrs = current_guest.vcpu_ptrs.lock();
        for vcpu_ptr in (*vcpu_ptrs).iter_mut() {
            if vcpu_ptr.is_none() {
                *vcpu_ptr = Some(cur_vcpu_ref);
                break;
            }
        }

        // Drop the lock here to prevent a deadlock.
        drop(vcpu_ptrs);

        // Update the global active hart counter.
        GUESTS[guest_id]
            .active_hart_count
            .fetch_sub(1, Ordering::Acquire);
        // Rotate array of guest IDs to start from the next guest.
        next_guest_vcpu = claim_guest(guest_id);

        // Fill the new vcpu with correct values for this hart.
        guest_id
    };

    unsafe {
        // Execute guest-specific routines, such as disabling unnecessary extensions
        // or marking them as "Dirty" if required to load the new guest's context.
        (*next_guest_vcpu).prepare_guest_vcpu_for_hart_switch(
            host_sp,
            host_hart_id,
            guest_id_for_hart,
        );
        (*next_guest_vcpu).run();
    }
}

fn claim_guest(current_guest_id: usize) -> *mut dyn VcpuInterface {
    let next_guest_vcpu: *mut dyn VcpuInterface;
    let mut guest_ids = GUEST_INDEXES;
    // Rotate the guest IDs array to start from the next sequential guest.
    guest_ids.rotate_left(current_guest_id);
    // Find an available guest among all guests in the system.
    'outer: loop {
        for guest_id in guest_ids {
            let vcpu_ptr_res = crate::guest::claim_vcpu_for_hart_if_available(guest_id);

            // If a vCPU was successfully claimed, assign it to this hart and exit the search.
            if let Ok(vcpu_ptr) = vcpu_ptr_res {
                next_guest_vcpu = vcpu_ptr;
                break 'outer;
            }
        }
    }
    next_guest_vcpu
}

fn claim_assigned_guest(
    guest_ids: &[Option<usize>],
    hart: &crate::multihart::Hart,
) -> (usize, *mut dyn VcpuInterface) {
    let next_guest_vcpu: *mut dyn VcpuInterface;
    let local_guest_id: usize;

    // Find an available guest among all assigned to this Hart guests.
    'outer: loop {
        for (id, guest_id) in guest_ids.iter().flatten().enumerate() {
            let is_claimed = crate::guest::claim_assigned_hart_slot_if_available(*guest_id);

            // Run a guest if it is free; otherwise, continue searching for a free guest.
            if is_claimed {
                next_guest_vcpu = unsafe { (*hart.assigned_guest_vcpus.get())[id] };
                local_guest_id = id;
                break 'outer;
            } else {
                continue;
            }
        }
    }
    (local_guest_id, next_guest_vcpu)
}
