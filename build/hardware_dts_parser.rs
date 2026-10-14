use std::collections::HashMap;

pub fn parse_dts(out_dir: &str) -> (Vec<[u64; 2]>, HashMap<Vec<String>, Vec<[u64; 2]>>) {
    let dtb_output_path = std::path::PathBuf::from(out_dir).join("hardware.dtb");

    // Compile Device Tree file.
    crate::build_modules::main_dts_parser::compile_dts(&dtb_output_path, "hardware.dts");

    // Parse Device Tree Binary file.
    let dtb_data = std::fs::read(dtb_output_path).unwrap();

    let fdt = fdt::Fdt::new(&dtb_data).unwrap();

    // The even-indexed array elements are the MMIO start addresses,
    // and the odd-indexed ones are the end sizes.
    let mut passthrough_memory_peripherals: Vec<[u64; 2]> = Vec::new();
    let mut aligned_passthrough_memory_peripherals: Vec<[u64; 2]> = Vec::new();
    let mut emulate_memory_peripherals: HashMap<Vec<String>, Vec<[u64; 2]>> = HashMap::new();

    if let Some(soc) = fdt.find_node("/soc") {
        for peripheral in soc.children() {
            let memory_policy_str = peripheral
                .property("memory-policy")
                .expect("No memory-policy field in the hardware DTS")
                .as_str()
                .unwrap();

            let compatible: Vec<String> = peripheral
                .property("compatible")
                .expect("No compatible field in the hardware DTS")
                .as_str()
                .unwrap()
                .split(",")
                .map(String::from)
                .collect();

            // Extract addresses of peripheral (reg).
            let mut reg_addrs: Vec<[u64; 2]> = Vec::new();

            let reg = peripheral
                .property("reg")
                .expect("No reg field in the hardware DTS");
            // Convert a raw `reg` DTB property into native array.
            let reg_raw = reg.value;

            if reg_raw.len() % 8 != 0 {
                panic!("reg cell is not a multiple of 8 bytes");
            }

            let (chunks, _remainder) = reg_raw.as_chunks::<8>();

            for bytes in chunks.chunks(2) {
                let addr: u64 = u64::from_be_bytes(bytes[0]);
                let size: u64 = u64::from_be_bytes(bytes[1]);
                reg_addrs.push([addr, size]);
            }

            if memory_policy_str == "emulate" {
                emulate_memory_peripherals.insert(compatible, reg_addrs);
            } else if memory_policy_str == "passthrough" {
                passthrough_memory_peripherals.extend(reg_addrs.iter());
            } else {
                panic!("Unsupported memory policy");
            }
        }
    }
    passthrough_memory_peripherals.sort_by_key(|chunk| chunk[0]);
    (passthrough_memory_peripherals, emulate_memory_peripherals)
}
