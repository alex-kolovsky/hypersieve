#![no_main]

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    safe fn claim_vcpu_for_hart_if_available(guest_id: usize) -> bool;
}
use hypersieve_abi::GUEST_INDEXES;

#[unsafe(no_mangle)]
pub fn claim_guest(current_guest_id: usize) -> usize {
    let mut guest_ids = GUEST_INDEXES;
    let id: usize;
    // Rotate the guest IDs array to start from the next sequential guest.
    guest_ids.rotate_left(current_guest_id);
    // Find an available guest among all guests in the system.
    'outer: loop {
        for guest_id in guest_ids {
            let is_claimed = claim_vcpu_for_hart_if_available(guest_id);

            // If a vCPU was successfully claimed, assign it to this hart and exit the search.
            if is_claimed {
                id = guest_id;
                break 'outer;
            }
        }
    }
    id
}
