use fdt::node::FdtNode;

use crate::build_modules::arch::{TargetArchFeatures, TargetArchGuestFeatures};

pub mod floating_point_extension_support;
pub mod guest_specific_routines;
pub mod vector_extension_support;

pub fn get_properties(
    target_arch_features: &mut TargetArchFeatures,
    guest_node: &FdtNode<'_, '_>,
    target_arch_guest_features: &mut TargetArchGuestFeatures,
) {
    // Sets VLEN and FP parameters for RISC-V 64 targets; defaults to zero/false otherwise.
    // Extract the vector length (VLEN).
    let vlen = guest_node
        .property("vlen")
        .expect("No vlen property in DTS")
        .as_usize()
        .unwrap();

    if vlen > target_arch_features.max_vlen {
        target_arch_features.max_vlen = vlen;
    }

    let fp = guest_node.property("floating-point-extension").is_some();

    if fp {
        target_arch_features.is_fp_needed = true;
    }

    target_arch_guest_features.vlen = vlen;
    target_arch_guest_features.fp = fp;
}
