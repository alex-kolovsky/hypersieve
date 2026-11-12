use fdt::node::FdtNode;

use crate::TargetArch;

mod riscv64;

#[derive(Debug, Default)]
pub struct TargetArchFeatures {
    pub max_vlen: usize,
    pub is_fp_needed: bool,
}

#[derive(Debug, Default)]
pub struct TargetArchGuestFeatures {
    pub vlen: usize,
    pub fp: bool,
}

pub fn generate_guest_specific_routine(
    target_arch: &TargetArch,
    guest_id: usize,
    target_arch_guest_features: &mut TargetArchGuestFeatures,
    guest_specific_routines_file: &mut std::fs::File,
) {
    if target_arch.is_riscv64() {
        riscv64::guest_specific_routines::generate_guest_specific_routine(
            guest_id,
            target_arch_guest_features,
            guest_specific_routines_file,
        );
    } else {
        unreachable!();
    }
}

pub fn parse_arch_related_values_from_dts(
    guest_node: &FdtNode<'_, '_>,
    target_arch_guest_features: &mut TargetArchGuestFeatures,
    target_arch: &TargetArch,
    target_arch_features: &mut TargetArchFeatures,
) {
    if target_arch.is_riscv64() {
        riscv64::get_properties(target_arch_features, guest_node, target_arch_guest_features);
    } else {
        unreachable!();
    }
}

pub fn generate_arch_support(
    out_dir: &str,
    target_arch: &TargetArch,
    target_arch_features: &TargetArchFeatures,
) {
    if target_arch.is_riscv64() {
        // Generate vector extension support file.
        riscv64::vector_extension_support::generate_vector_extension_support(
            out_dir,
            target_arch_features.max_vlen,
        );

        // Generate floating-point extension support file.
        riscv64::floating_point_extension_support::generate_floating_point_extension_support(
            out_dir,
            target_arch_features.is_fp_needed,
        );
    } else {
        unreachable!();
    }
}

pub fn get_driver_config(target_arch: &TargetArch) -> String {
    if target_arch.is_riscv64() {
        std::fs::read_to_string(concat!(
            crate::hypersieve_root_dir!(),
            "src/arch/riscv64/mmio/drivers.toml"
        ))
        .unwrap()
    } else {
        unreachable!();
    }
}
