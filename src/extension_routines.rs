use crate::vcpu::Vcpu;

pub fn restore_vrs_if_needed(vcpu_ptr: *mut Vcpu) {
    let vcpu_ref = unsafe { &(*vcpu_ptr) };
    if (vcpu_ref.vsstatus & ((1 << 9) | (1 << 10))) != 0 {
        // Restore vector registers.
        crate::restore_vrs(vcpu_ptr);
    }
}
pub fn restore_fprs_if_needed(vcpu_ptr: *mut Vcpu) {
    let vcpu_ref = unsafe { &(*vcpu_ptr) };
    if (vcpu_ref.vsstatus & ((1 << 14) | (1 << 13))) != 0 {
        // Restore floating-point registers.
        crate::restore_fprs(vcpu_ptr);
    }
}

pub fn save_vrs_if_needed(vcpu_ptr: *mut Vcpu) {
    let vcpu_ref = unsafe { &(*vcpu_ptr) };
    if (vcpu_ref.vsstatus & ((1 << 9) | (1 << 10))) != 0 {
        // Save vector registers.
        crate::save_vrs(vcpu_ptr);
    }
}

pub fn save_fprs_if_needed(vcpu_ptr: *mut Vcpu) {
    let vcpu_ref = unsafe { &(*vcpu_ptr) };
    if (vcpu_ref.vsstatus & ((1 << 14) | (1 << 13))) != 0 {
        // Save floating-point registers.
        crate::save_fprs(vcpu_ptr);
    }
}
