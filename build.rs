use std::{collections::HashMap, io::Write};

struct Guest {
    entry_gpa: usize,
    harts_cap: usize,
    dedicated_harts: Option<Vec<u32>>,
}
impl Guest {
    fn new(entry_gpa: usize, harts_cap: usize, dedicated_harts: Option<Vec<u32>>) -> Self {
        Self {
            entry_gpa,
            harts_cap,
            dedicated_harts,
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=config.dts");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile Device Tree file
    let dtb_output_path = std::path::PathBuf::from(&out_dir).join("config.dtb");
    let dts_source_path = "config.dts";

    let dtc_output = std::process::Command::new("dtc")
        .arg("-I")
        .arg("dts")
        .arg("-O")
        .arg("dtb")
        .arg("-o")
        .arg(&dtb_output_path)
        .arg(dts_source_path)
        .output()
        .expect("Failed to execute 'dtc'. Is device-tree-compiler installed on your system?");

    if !dtc_output.status.success() {
        panic!(
            "DTC Compilation Error: {}",
            String::from_utf8_lossy(&dtc_output.stderr)
        );
    }

    // Parse Device Tree Binary file.
    let dtb_data = std::fs::read(dtb_output_path).unwrap();

    let mut max_supported_harts_per_guest: usize = 0;
    let mut max_supported_dedicated_harts_per_guest: usize = 0;

    let mut harts_cap_sum: usize = 0;

    let mut already_dedicated_harts: HashMap<u32, usize> = HashMap::new();

    let mut paths: Vec<&str> = Vec::new();

    let mut guests: Vec<Guest> = Vec::new();
    let fdt = fdt::Fdt::new(&dtb_data).unwrap();
    if let Some(hypervisor) = fdt.find_node("/hypervisor-config") {
        for guest in hypervisor.children() {
            // Extract VM entry address.
            let entry = guest
                .property("entry")
                .expect("No entry field in DTS")
                .as_usize()
                .expect("Entry is not an integer");
            // Extract maximum number of harts for VM.
            let harts_cap = guest
                .property("harts-cap")
                .expect("No harts-cap field in DTS")
                .as_usize()
                .unwrap();

            let path = guest
                .property("path")
                .expect("No path field in DTS")
                .as_str()
                .unwrap();

            paths.push(path);

            harts_cap_sum += harts_cap;

            // Find the maximum number of harts among all VMs.
            if harts_cap > max_supported_harts_per_guest {
                max_supported_harts_per_guest = harts_cap;
            }

            // Extract dedicated harts
            if let Some(dedicated_harts) = guest.property("dedicated-harts") {
                // Convert a raw `dedicated-harts` DTB property into native CPU Hart IDs.
                let dedicated_harts_raw = dedicated_harts.value;

                if dedicated_harts_raw.len() % 4 != 0 {
                    panic!("dedicated-harts cell is not a multiple of 4 bytes");
                }

                let mut harts = Vec::new();
                let (chunks, _remainder) = dedicated_harts_raw.as_chunks::<4>();

                for bytes in chunks.iter() {
                    let hart_id: u32 = u32::from_be_bytes(*bytes);
                    harts.push(hart_id);
                }
                if harts.len() > harts_cap {
                    panic!(
                        "Dedicated harts count ({}) cannot be bigger than harts capacity ({harts_cap})\nGuest name: {}",
                        harts.len(),
                        guest.name,
                    );
                }

                // Find the maximum number of dedicated harts among all VMs.
                if harts.len() > max_supported_dedicated_harts_per_guest {
                    max_supported_dedicated_harts_per_guest = harts.len();
                }

                for hart in &harts {
                    if let Some(already_dedicated_hart) = already_dedicated_harts.get_mut(hart) {
                        *already_dedicated_hart += 1;
                    } else {
                        already_dedicated_harts.insert(*hart, 1);
                    }
                }

                let guest_entry = Guest::new(entry, harts_cap, Some(harts));
                guests.push(guest_entry);
            } else {
                let guest_entry = Guest::new(entry, harts_cap, None);
                guests.push(guest_entry);
            }
        }
    }

    let mut max_supported_dedicated_guests_per_hart: usize = 0;

    for times_dedicated in already_dedicated_harts.values() {
        if *times_dedicated > max_supported_dedicated_guests_per_hart {
            max_supported_dedicated_guests_per_hart = *times_dedicated;
        }
    }

    // Create guests.rs file.
    let dest_path = std::path::Path::new(&out_dir).join("guests.rs");
    let mut f = std::fs::File::create(&dest_path).unwrap();

    writeln!(
        f,
        "// Automaticly generated by build.rs. Do not edit.

pub const MAX_SUPPORTED_DEDICATED_GUESTS_PER_HART: usize = {max_supported_dedicated_guests_per_hart};
pub const MAX_SUPPORTED_HARTS_PER_GUEST: usize = {max_supported_harts_per_guest};
pub const MAX_SUPPORTED_DEDICATED_HARTS_PER_GUEST: usize = {max_supported_dedicated_harts_per_guest};

pub const HARTS_CAP_SUM: usize = {harts_cap_sum};

pub static GUESTS: [Guest; {}] = [
",
        guests.len(),
    )
    .unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    for (i, guest) in guests.iter().enumerate() {
        // Create guest.vcpus array.
        let mut vcpus: String = String::from("[");
        for vcpus_i in 0..max_supported_harts_per_guest {
            if vcpus_i < guest.harts_cap {
                vcpus.push_str("Some(crate::vcpu::Vcpu::zeroed()), ");
            } else {
                vcpus.push_str("None, ");
            }
        }
        vcpus.push(']');

        let dedicated_harts_count: usize =
            if let Some(dedicated_harts) = guest.dedicated_harts.as_ref() {
                dedicated_harts.len()
            } else {
                0
            };

        // Add guest entry in GUESTS array.
        writeln!(
            f,
            "Guest {{
    entry_gpa: {},
    harts: core::sync::atomic::AtomicUsize::new({dedicated_harts_count}),
    harts_cap: {},
    dedicated_harts: {:#?},
    data: include_bytes!(\"{manifest_dir}/{}\"),
    vcpus: core::cell::UnsafeCell::new({vcpus}),
    vcpu_ptrs: core::cell::UnsafeCell::new([const {{ None }}; MAX_SUPPORTED_HARTS_PER_GUEST]),
}},

",
            guest.entry_gpa, guest.harts_cap, guest.dedicated_harts, paths[i]
        )
        .unwrap();
    }

    writeln!(f, "];").unwrap();
}
