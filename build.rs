use crate::build_modules::{
    arch::TargetArchGuestFeatures,
    hardware_dts_parser::{MemoryRegion, MmioPeripheral},
};

pub static SUPPORTED_ARCHS: [&str; 1] = ["riscv64"];

#[path = "build/mod.rs"]
mod build_modules;

#[derive(Debug)]
struct Guest {
    entry_gpa: usize,
    hart_capacity: usize,
    assigned_harts: Vec<Option<u32>>,
    path: String,
    aligned_passthrough_mmio_regions: Vec<MemoryRegion>,
    allowed_mmio_peripherals: Vec<MmioPeripheral>,
    target_arch_guest_features: TargetArchGuestFeatures,
}

#[derive(Debug, Default)]
pub struct Hart {
    pub hart_id: usize,
    pub guests: Vec<Option<usize>>,
}

pub struct TargetArch {
    name: String,
}
impl TargetArch {
    pub fn is_riscv64(&self) -> bool {
        self.name == "riscv64"
    }
}

fn main() {
    println!("cargo:rerun-if-changed=config.dts");
    println!("cargo:rerun-if-changed=hardware.dts");
    println!("cargo:rerun-if-changed=arch/riscv/mmio/driver.rs");

    let target_arch = TargetArch {
        name: std::env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
    };

    if !SUPPORTED_ARCHS.contains(&target_arch.name.as_str()) {
        panic!("{} architecture is not supported.", target_arch.name);
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile and parse the hardware device tree file.
    let memory_peripherals = build_modules::hardware_dts_parser::parse_dts(out_dir.as_str());

    // Compile and parse the main device tree file.
    let mut hypervisor_configuration = build_modules::main_dts_parser::parse_dts(
        out_dir.as_str(),
        &target_arch,
        memory_peripherals,
    );

    build_modules::arch::generate_arch_support(
        &out_dir,
        &target_arch,
        &hypervisor_configuration.target_arch_features,
    );

    // Parse the driver configuration file.
    let drivers_config = crate::build_modules::drivers::parse(&target_arch);

    // Generate guest constants file.
    build_modules::guest_constants::generate_guest_constants(
        out_dir.as_str(),
        &mut hypervisor_configuration,
        &target_arch,
        drivers_config,
    );

    // Generate hart constants file.
    build_modules::hart_constants::generate_hart_constants(
        &out_dir,
        hypervisor_configuration.harts,
        hypervisor_configuration.total_hart_capacity,
    );
}
