#![no_std]

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    safe fn claim_vcpu_for_hart_if_available(guest_id: usize) -> bool;
}
use hypersieve_abi::GUEST_INDEXES;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unreachable!();
}

#[unsafe(no_mangle)]
pub fn claim_guest(current_guest_id: usize) -> usize {
    let guest_ids = GUEST_INDEXES;
    let len = guest_ids.len();

    // Get the ID of the sequential guest.
    let next_guest_id = if current_guest_id == len {
        0
    } else {
        current_guest_id + 1
    };

    // Find an available guest among all guests.
    loop {
        // Rotate the guest IDs array to start from the next sequential guest.
        for guest_id in (next_guest_id..len).chain(0..next_guest_id) {
            let is_claimed = claim_vcpu_for_hart_if_available(guest_id);

            // Return ID of the guest if it is free; otherwise, continue searching for a free guest.
            if is_claimed {
                return guest_id;
            }
        }
    }
}
