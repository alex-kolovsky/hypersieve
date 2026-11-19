#![no_std]

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    safe fn claim_assigned_hart_slot_if_available(guest_id: usize) -> bool;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable();
}

use hypersieve_abi::ASSIGNED_GUESTS_BY_HART_ID;

#[unsafe(no_mangle)]
pub fn claim_assigned_guest(current_guest_id: usize, hart_id: usize) -> usize {
    let guest_ids = ASSIGNED_GUESTS_BY_HART_ID[hart_id];
    let len = guest_ids.len();

    // Get the ID of the sequential guest.
    let next_guest_id = if current_guest_id == len {
        0
    } else {
        current_guest_id + 1
    };

    // Find an available guest among all assigned to this Hart guests.
    'outer: loop {
        // Rotate the assigned guest IDs array to start from the next sequential guest.
        for local_guest_id in (next_guest_id..len).chain(0..next_guest_id) {
            if let Some(guest_id) = guest_ids[local_guest_id] {
                let is_claimed = claim_assigned_hart_slot_if_available(guest_id);

                // Return ID of the guest if it is free; otherwise, continue searching for a free guest.
                if is_claimed {
                    return local_guest_id;
                } else {
                    continue;
                }
            } else {
                continue 'outer;
            }
        }
    }
}
