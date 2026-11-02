use crate::arch::Vcpu;
use crate::arch::riscv::{restore_fprs, restore_vrs, save_fprs, save_vrs};

pub fn restore_vrs_if_needed(vcpu_ref: &mut Vcpu) {
    if (vcpu_ref.vsstatus & ((1 << 9) | (1 << 10))) != 0 {
        // Restore vector registers.
        restore_vrs(vcpu_ref);
    }
}
pub fn restore_fprs_if_needed(vcpu_ref: &mut Vcpu) {
    if (vcpu_ref.vsstatus & ((1 << 14) | (1 << 13))) != 0 {
        // Restore floating-point registers.
        restore_fprs(vcpu_ref);
    }
}

pub fn save_vrs_if_needed(vcpu_ref: &mut Vcpu) {
    if (vcpu_ref.vsstatus & ((1 << 9) | (1 << 10))) != 0 {
        // Save vector registers.
        save_vrs(vcpu_ref);
    }
}

pub fn save_fprs_if_needed(vcpu_ref: &mut Vcpu) {
    if (vcpu_ref.vsstatus & ((1 << 14) | (1 << 13))) != 0 {
        // Save floating-point registers.
        save_fprs(vcpu_ref);
    }
}

pub fn set_fs(vcpu_ref: &mut Vcpu, fs: u64) {
    // Clean FS bits.
    vcpu_ref.vsstatus &= !((1 << 14) | (1 << 13));
    // Set FS to the given value.
    vcpu_ref.vsstatus |= fs << 13;
}

pub fn set_vs(vcpu_ref: &mut Vcpu, vs: u64) {
    // Clean VS bits.
    vcpu_ref.vsstatus &= !((1 << 9) | (1 << 10));
    // Set VS to the given value.
    vcpu_ref.vsstatus |= vs << 9;
}
