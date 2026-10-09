#[path = "build/mod.rs"]
mod build_modules;

#[derive(Debug)]
struct Guest {
    entry_gpa: usize,
    hart_capacity: usize,
    assigned_harts: Vec<Option<u32>>,
    path: String,
    passthrough_mmio_addrs: Vec<u64>,
}
impl Guest {
    fn new(
        entry_gpa: usize,
        hart_capacity: usize,
        assigned_harts: Vec<Option<u32>>,
        path: String,
        passthrough_mmio_addrs: Vec<u64>,
    ) -> Self {
        Self {
            entry_gpa,
            hart_capacity,
            assigned_harts,
            path,
            passthrough_mmio_addrs,
        }
    }
}

#[derive(Debug, Default)]
pub struct Hart {
    pub hart_id: usize,
    pub guests: Vec<Option<usize>>,
}

fn main() {
    println!("cargo:rerun-if-changed=config.dts");
    println!("cargo:rerun-if-changed=hardware.dts");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile and parse the hardware device tree file.
    let (passthrough_memory_peripherals, _emulate_memory_peripherals) =
        build_modules::hardware_dts_parser::parse_dts(out_dir.as_str());

    // Compile and parse the main device tree file.
    let mut hypervisor_configuration =
        build_modules::main_dts_parser::parse_dts(out_dir.as_str(), passthrough_memory_peripherals);

    // Generate vector extension support file.
    build_modules::vector_extension_support::generate_vector_extension_support(
        &out_dir,
        hypervisor_configuration.vlen,
    );

    // Generate floating-point extension support file.
    build_modules::floating_point_extension_support::generate_floating_point_extension_support(
        &out_dir,
        hypervisor_configuration.floating_point_extension,
    );

    // Generate guest constants file.
    build_modules::guest_constants::generate_guest_constants(
        out_dir.as_str(),
        &mut hypervisor_configuration.guests,
        hypervisor_configuration.max_supported_harts_per_guest,
        hypervisor_configuration.max_supported_assigned_harts_per_guest,
    );

    // Generate hart constants file.
    build_modules::hart_constants::generate_hart_constants(
        &out_dir,
        hypervisor_configuration.harts,
        hypervisor_configuration.total_hart_capacity,
    );
}
