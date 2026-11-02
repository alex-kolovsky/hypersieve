use crate::arch::Vcpu;

pub fn handle_trap(vcpu: *mut Vcpu) {
    let vcpu_ref = unsafe { &mut (*vcpu) };

    // Execute the guest-specific trap routine, such as saving extension registers.
    let guest = unsafe { &*vcpu_ref.guest };
    let guest_specific_trap_routine = guest.guest_specific_trap_routine;
    guest_specific_trap_routine(vcpu_ref);

    // Process an exception.
    crate::arch::process_exception(vcpu_ref);
}
