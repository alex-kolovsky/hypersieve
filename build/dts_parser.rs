use crate::Guest;
use crate::Hart;

pub struct HypervisorConfiguration {
    pub guests: Vec<Guest>,
    pub harts: Vec<Hart>,

    pub total_hart_capacity: usize,

    pub max_supported_harts_per_guest: usize,
    pub max_supported_assigned_harts_per_guest: usize,

    pub vlen: usize,
}

pub fn compile_dts(dtb_output_path: &std::path::PathBuf) {
    let dts_source_path = "config.dts";

    let dtc_output = std::process::Command::new("dtc")
        // Input format.
        .arg("-I")
        .arg("dts")
        // Output format.
        .arg("-O")
        .arg("dtb")
        // Output path.
        .arg("-o")
        .arg(dtb_output_path)
        // Input path.
        .arg(dts_source_path)
        .output()
        .expect("Failed to execute 'dtc'. Is device-tree-compiler installed on your system?");

    if !dtc_output.status.success() {
        panic!(
            "DTC Compilation Error: {}",
            String::from_utf8_lossy(&dtc_output.stderr)
        );
    }
}

pub fn parse_dts(out_dir: &str) -> HypervisorConfiguration {
    let dtb_output_path = std::path::PathBuf::from(out_dir).join("config.dtb");

    // Compile Device Tree file.
    compile_dts(&dtb_output_path);

    // Parse Device Tree Binary file.
    let dtb_data = std::fs::read(dtb_output_path).unwrap();

    // A vector of all assigned harts in a system.
    let mut harts: Vec<Hart> = Vec::new();

    let mut max_supported_harts_per_guest: usize = 0;
    let mut max_supported_assigned_harts_per_guest: usize = 0;

    // Total hart capacity allocated across all guest VMs.
    let mut total_hart_capacity: usize = 0;

    let vlen: usize;

    let mut guests: Vec<Guest> = Vec::new();
    let fdt = fdt::Fdt::new(&dtb_data).unwrap();
    if let Some(hypervisor) = fdt.find_node("/hypervisor-config") {
        // Extract the vector length (VLEN).
        vlen = hypervisor
            .property("vlen-size")
            .expect("No vlen-size property in DTS")
            .as_usize()
            .unwrap();

        for guest in hypervisor.children() {
            // Extract the VM entry address.
            let entry = guest
                .property("entry")
                .expect("No entry field in DTS")
                .as_usize()
                .expect("Entry is not an integer");
            // Extract the hart capacity for the VM.
            let hart_capacity = guest
                .property("hart-capacity")
                .expect("No hart-capacity field in DTS")
                .as_usize()
                .unwrap();

            let path = guest
                .property("path")
                .expect("No path field in DTS")
                .as_str()
                .unwrap();

            if !std::path::Path::exists(std::path::Path::new(path)) {
                panic!("Guest file {path} not found");
            }

            total_hart_capacity += hart_capacity;

            // Find the maximum hart capacity among all VMs.
            if hart_capacity > max_supported_harts_per_guest {
                max_supported_harts_per_guest = hart_capacity;
            }

            // Extract assigned harts.
            if let Some(guest_assigned_harts) = guest.property("assigned-harts") {
                // Convert a raw `assigned-harts` DTB property into native CPU Hart IDs.
                let guest_assigned_harts_raw = guest_assigned_harts.value;

                if guest_assigned_harts_raw.len() % 4 != 0 {
                    panic!("assigned-harts cell is not a multiple of 4 bytes");
                }

                let mut guest_assigned_harts = Vec::new();
                let (chunks, _remainder) = guest_assigned_harts_raw.as_chunks::<4>();

                for bytes in chunks.iter() {
                    let hart_id: u32 = u32::from_be_bytes(*bytes);
                    guest_assigned_harts.push(Some(hart_id));
                    max_supported_assigned_harts_per_guest += 1;
                }
                if harts.len() > hart_capacity {
                    panic!(
                        "Assigned harts count ({}) cannot be bigger than harts capacity ({hart_capacity})\nGuest name: {}",
                        harts.len(),
                        guest.name,
                    );
                }

                // Find the maximum number of assigned harts among all VMs.
                if harts.len() > max_supported_assigned_harts_per_guest {
                    max_supported_assigned_harts_per_guest = harts.len();
                }

                let guest_entry = Guest::new(
                    entry,
                    hart_capacity,
                    guest_assigned_harts,
                    String::from(path),
                );

                guests.push(guest_entry);
            } else {
                let guest_entry = Guest::new(entry, hart_capacity, Vec::new(), String::from(path));
                guests.push(guest_entry);
            }
        }
    } else {
        panic!("No hypervisor configuration in DTS");
    }

    // Populate Hart structures for the harts vector.
    for hart_id in 0..total_hart_capacity {
        let entry: Hart = Hart {
            hart_id,
            guests: Vec::new(),
        };
        harts.push(entry);
    }

    // Populate assigned guests for harts.
    for (guest_id, guest) in guests.iter().enumerate() {
        for hart in guest.assigned_harts.iter().flatten() {
            harts[*hart as usize].guests.push(Some(guest_id));
        }
    }

    HypervisorConfiguration {
        guests,
        max_supported_harts_per_guest,
        max_supported_assigned_harts_per_guest,
        harts,
        total_hart_capacity,
        vlen,
    }
}
