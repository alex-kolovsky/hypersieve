#![no_main]

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    safe fn claim_assigned_hart_slot_if_available(guest_id: usize) -> bool;
}
use hypersieve_abi::ASSIGNED_GUESTS_BY_HART_ID;

#[unsafe(no_mangle)]
pub fn claim_assigned_guest(current_guest_id: usize, hart_id: usize) -> usize {
    // Rotate the assigned guest IDs array to start from the next sequential guest.
    let mut guest_ids = ASSIGNED_GUESTS_BY_HART_ID[hart_id];
    guest_ids.rotate_left(current_guest_id);

    let local_guest_id: usize;

    // Find an available guest among all assigned to this Hart guests.
    'outer: loop {
        for (id, guest_id) in guest_ids.iter().flatten().enumerate() {
            let is_claimed = claim_assigned_hart_slot_if_available(*guest_id);

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
