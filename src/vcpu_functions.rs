use crate::{GUESTS, arch::VcpuInterface};
use core::sync::atomic::Ordering;

pub fn run_next_guest(cur_vcpu_ref: &'static mut dyn VcpuInterface) {
    // Save these values before releasing this vcpu
    // to other harts to prevent data races.
    let host_sp = cur_vcpu_ref.host_stack_pointer();
    let host_hart_id = cur_vcpu_ref.host_hart_id();

    let current_guest = cur_vcpu_ref.current_guest_ref();
    let current_guest_id_for_hart = cur_vcpu_ref.current_guest_id_for_hart();
    let guest_id = current_guest.info.id;

    let is_assigned = crate::HARTS[host_hart_id].is_assigned;

    let next_guest_vcpu: *mut dyn VcpuInterface;

    let guest_id_for_hart = if is_assigned {
        // Decrement the global counter of active harts for this guest,
        // allowing another waiting hart to claim and run it.
        current_guest
            .info
            .active_assigned_hart_count
            .fetch_sub(1, Ordering::Acquire);

        let hart = &crate::HARTS[cur_vcpu_ref.host_hart_id()];

        let vcpu_ptrs = hart.assigned_guest_vcpus.get();

        // Store current guest VCPU reference into the local VCPUs array to use it in later switches.
        unsafe {
            (*vcpu_ptrs)[current_guest_id_for_hart] = Some(cur_vcpu_ref);
        }

        // Rotate the assigned guest IDs array to start from the next sequential guest.
        let mut guest_ids = *hart.guests;
        guest_ids.rotate_left(guest_id);

        // Search for an available guest and claim it for this hart.
        let local_guest_id: usize = claim_assigned_guest(&guest_ids);
        let vcpu_ptrs = unsafe { &mut *hart.assigned_guest_vcpus.get() };
        next_guest_vcpu = vcpu_ptrs[local_guest_id].take().unwrap();
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
        current_guest
            .info
            .active_hart_count
            .fetch_sub(1, Ordering::Acquire);

        let new_guest = &GUESTS[crate::wasm::Modules::claim_guest(guest_id) as usize];
        next_guest_vcpu = get_global_vcpu_ptr(new_guest);

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
fn claim_assigned_guest(guest_ids: &[Option<usize>]) -> usize {
    let local_guest_id: usize;

    // Find an available guest among all assigned to this Hart guests.
    'outer: loop {
        for (id, guest_id) in guest_ids.iter().flatten().enumerate() {
            let is_claimed = crate::guest::claim_assigned_hart_slot_if_available(*guest_id);

            // Run a guest if it is free; otherwise, continue searching for a free guest.
            if is_claimed {
                local_guest_id = id;
                break 'outer;
            } else {
                continue;
            }
        }
    }
    local_guest_id
}

pub fn get_global_vcpu_ptr(guest: &crate::guest::Guest) -> &mut dyn VcpuInterface {
    let mut vcpu_ptrs = guest.vcpu_ptrs.lock();
    let vcpu_ptr = vcpu_ptrs[0].take().unwrap();
    vcpu_ptrs[0] = None;
    vcpu_ptrs.rotate_left(1);
    drop(vcpu_ptrs);
    vcpu_ptr
}
