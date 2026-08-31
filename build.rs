#[path = "build/mod.rs"]
mod build_modules;

struct Guest {
    entry_gpa: usize,
    hart_capacity: usize,
    assigned_harts: Vec<Option<u32>>,
    path: String,
}
impl Guest {
    fn new(
        entry_gpa: usize,
        hart_capacity: usize,
        assigned_harts: Vec<Option<u32>>,
        path: String,
    ) -> Self {
        Self {
            entry_gpa,
            hart_capacity,
            assigned_harts,
            path,
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

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile and parse the device tree file.
    let mut hypervisor_configuration = build_modules::dts_parser::parse_dts(out_dir.as_str());

    // Generate vector extension support file.
    build_modules::vector_extension_support::generate_vector_extension_support(
        &out_dir,
        hypervisor_configuration.vlen,
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
