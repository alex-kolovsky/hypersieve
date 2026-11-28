use crate::build_modules::{
    config_dts_parser::HypervisorConfiguration,
    guest_constants::generate_allowed_buffer_infos_string,
};
use std::io::Write;

pub fn generate_wasm_guest_constants(
    out_dir: &str,
    hypervisor_configuration: &HypervisorConfiguration,
) {
    let wasm_guest_constants_file_path =
        std::path::Path::new(out_dir).join("wasm_guest_constants.rs");
    let mut wasm_guest_constants_file = std::fs::File::create(wasm_guest_constants_file_path)
        .expect("[FATAL][BUILD]: failed to create a file");

    writeln!(
        wasm_guest_constants_file,
        "
pub static WASM_GUESTS: [crate::guest::WasmGuest; {}] = [
        ",
        hypervisor_configuration.wasm_guests.len(),
    )
    .expect("[FATAL][BUILD]: failed to perform a write to the file");

    for wasm_guest in &hypervisor_configuration.wasm_guests {
        let temp_vec: Vec<u32> = vec![];
        let allowed_to_read_shared_buffers_vec = wasm_guest
            .allowed_to_read_shared_buffers
            .as_ref()
            .unwrap_or(&temp_vec);
        let allowed_to_write_shared_buffers_vec = wasm_guest
            .allowed_to_write_shared_buffers
            .as_ref()
            .unwrap_or(&temp_vec);

        let allowed_buffer_infos = generate_allowed_buffer_infos_string(
            &hypervisor_configuration.shared_buffers,
            hypervisor_configuration.max_allowed_shared_buffer_count_per_guest,
            allowed_to_write_shared_buffers_vec,
            allowed_to_read_shared_buffers_vec,
        );

        writeln!(
            wasm_guest_constants_file,
            "
crate::guest::WasmGuest {{
    shared_buffer_infos: {allowed_buffer_infos},
}},
            "
        )
        .expect("[FATAL][BUILD]: failed to perform a write to the file");
    }

    writeln!(wasm_guest_constants_file, "];")
        .expect("[FATAL][BUILD]: failed to perform a write to the file");
}
