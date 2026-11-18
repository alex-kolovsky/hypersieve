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

    // Rotate the guest IDs array to start from the next sequential guest.
    let next_guest_id = if current_guest_id == len {
        0
    } else {
        current_guest_id + 1
    };

    // Find an available guest among all assigned to this Hart guests.
    loop {
        for guest_id in (next_guest_id..len).chain(0..next_guest_id) {
            let is_claimed = claim_vcpu_for_hart_if_available(guest_id);

            // If a vCPU was successfully claimed, assign it to this hart and exit the search.
            if is_claimed {
                return guest_id;
            }
        }
    }
}
