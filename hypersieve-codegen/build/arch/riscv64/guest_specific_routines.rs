use std::io::Write;

use crate::build_modules::arch::TargetArchGuestFeatures;

pub fn generate_guest_specific_routine(
    guest_id: usize,
    target_arch_guest_features: &TargetArchGuestFeatures,
    guest_specific_routines_file: &mut std::fs::File,
) {
    let mut trap_routine_function =
        format!("fn guest_{guest_id}_trap_routine(vcpu_ref: &mut Vcpu) {{\n");
    let mut ret_routine_function =
        format!("fn guest_{guest_id}_ret_routine(vcpu_ref: &mut Vcpu) {{\n");
    let mut switch_routine_function =
        format!("fn guest_{guest_id}_switch_routine(vcpu_ref: &mut Vcpu) {{\n");

    if target_arch_guest_features.vlen != 0 {
        trap_routine_function.push_str("extension_routines::save_vrs_if_needed(vcpu_ref);\n");
        ret_routine_function.push_str("extension_routines::restore_vrs_if_needed(vcpu_ref);\n");

        // Set VS bit to dirty, so the hypervisor will restore VRs with new guest registers.
        switch_routine_function.push_str("extension_routines::set_vs(vcpu_ref, 0b11);\n");
    } else {
        // Set VS bit to OFF, so the hypervisor will turn off vector registers for this hart.
        switch_routine_function.push_str("extension_routines::set_vs(vcpu_ref, 0b00);\n");
    }
    if target_arch_guest_features.fp {
        trap_routine_function.push_str("extension_routines::save_fprs_if_needed(vcpu_ref);\n");
        ret_routine_function.push_str("extension_routines::restore_fprs_if_needed(vcpu_ref);\n");

        // Set FS bit to dirty, so the hypervisor will restore FPRs with new guest registers.
        switch_routine_function.push_str("extension_routines::set_fs(vcpu_ref, 0b11);\n");
    } else {
        // Set FS bit to OFF, so the hypervisor will turn off floating point registers for this hart.
        switch_routine_function.push_str("extension_routines::set_fs(vcpu_ref, 0b00);\n");
    }

    trap_routine_function.push_str("}\n");
    ret_routine_function.push_str("}\n");
    switch_routine_function.push_str("}\n");

    write!(
        guest_specific_routines_file,
        "{ret_routine_function}
{trap_routine_function}
{switch_routine_function}"
    )
    .unwrap();
}
