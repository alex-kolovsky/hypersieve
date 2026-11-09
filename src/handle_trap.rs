use crate::arch::{Vcpu, VcpuInterface};

pub fn handle_trap(vcpu_ref: &'static mut Vcpu) {
    // Execute the guest-specific trap routine, such as saving extension registers.
    let guest = vcpu_ref.current_guest_ref();
    let guest_specific_trap_routine = guest.guest_specific_trap_routine;
    guest_specific_trap_routine(vcpu_ref);

    // Process an exception.
    crate::arch::process_exception(vcpu_ref);
}
