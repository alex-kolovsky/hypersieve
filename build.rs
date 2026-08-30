use std::{collections::HashMap, io::Write};

struct Guest {
    entry_gpa: usize,
    harts_cap: usize,
    assigned_harts: Vec<Option<u32>>,
}
impl Guest {
    fn new(entry_gpa: usize, harts_cap: usize, assigned_harts: Vec<Option<u32>>) -> Self {
        Self {
            entry_gpa,
            harts_cap,
            assigned_harts,
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

    let mut all_harts: Vec<Hart> = Vec::new();
    let mut max_supported_harts_per_guest: usize = 0;
    let mut max_supported_assigned_harts_per_guest: usize = 0;

    let mut harts_cap_sum: usize = 0;

    let mut already_assigned_harts: HashMap<u32, Vec<Option<usize>>> = HashMap::new();

    let mut paths: Vec<&str> = Vec::new();

    let mut guests: Vec<Guest> = Vec::new();
    let fdt = fdt::Fdt::new(&dtb_data).unwrap();
    if let Some(hypervisor) = fdt.find_node("/hypervisor-config") {
        for (guest_index, guest) in hypervisor.children().enumerate() {
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

            if !std::path::Path::exists(std::path::Path::new(path)) {
                panic!("Guest file {path} not found.");
            }

            harts_cap_sum += harts_cap;

            // Find the maximum number of harts among all VMs.
            if harts_cap > max_supported_harts_per_guest {
                max_supported_harts_per_guest = harts_cap;
            }

            // Extract assigned harts.
            if let Some(assigned_harts) = guest.property("assigned-harts") {
                // Convert a raw `assigned-harts` DTB property into native CPU Hart IDs.
                let assigned_harts_raw = assigned_harts.value;

                if assigned_harts_raw.len() % 4 != 0 {
                    panic!("assigned-harts cell is not a multiple of 4 bytes");
                }

                let mut harts = Vec::new();
                let (chunks, _remainder) = assigned_harts_raw.as_chunks::<4>();

                for bytes in chunks.iter() {
                    let hart_id: u32 = u32::from_be_bytes(*bytes);
                    harts.push(Some(hart_id));
                }
                if harts.len() > harts_cap {
                    panic!(
                        "Assigned harts count ({}) cannot be bigger than harts capacity ({harts_cap})\nGuest name: {}",
                        harts.len(),
                        guest.name,
                    );
                }

                // Find the maximum number of assigned harts among all VMs.
                if harts.len() > max_supported_assigned_harts_per_guest {
                    max_supported_assigned_harts_per_guest = harts.len();
                }

                // Increment the assignment counter for each assigned hart.
                for hart in &harts {
                    if let Some(already_assigned_hart) =
                        already_assigned_harts.get_mut(&hart.unwrap())
                    {
                        already_assigned_hart.push(Some(guest_index));
                    } else {
                        already_assigned_harts.insert(hart.unwrap(), vec![Some(guest_index)]);
                    }
                }

                let guest_entry = Guest::new(entry, harts_cap, harts);
                guests.push(guest_entry);
            } else {
                let guest_entry = Guest::new(entry, harts_cap, Vec::new());
                guests.push(guest_entry);
            }
        }
    }

    let mut max_supported_assigned_guests_per_hart: usize = 0;

    for times_assigned in already_assigned_harts.values() {
        if times_assigned.len() > max_supported_assigned_guests_per_hart {
            max_supported_assigned_guests_per_hart = times_assigned.len();
        }
    }

    for hart_id in 0..harts_cap_sum {
        if let Some(assigned_guests) = already_assigned_harts.get(&(hart_id as u32)) {
            let entry: Hart = Hart {
                hart_id,
                guests: Vec::new(),
            };
            for _ in 0..(max_supported_assigned_guests_per_hart - assigned_guests.len()) {}
            all_harts.push(entry);
        } else {
            let entry = Hart {
                hart_id,
                guests: Vec::new(),
            };
            all_harts.push(entry);
        }
    }

    // Create guests.rs file.
    let dest_path = std::path::Path::new(&out_dir).join("guests.rs");
    let mut f = std::fs::File::create(&dest_path).unwrap();

    writeln!(
        f,
        "// Automaticly generated by build.rs. Do not edit.
use crate::multihart::Hart;

pub const MAX_SUPPORTED_ASSIGNED_GUESTS_PER_HART: usize = {max_supported_assigned_guests_per_hart};
pub const MAX_SUPPORTED_HARTS_PER_GUEST: usize = {max_supported_harts_per_guest};
pub const MAX_SUPPORTED_ASSIGNED_HARTS_PER_GUEST: usize = {max_supported_assigned_harts_per_guest};

pub const HARTS_CAP_SUM: usize = {harts_cap_sum};

pub static GUESTS: [Guest; {}] = [
",
        guests.len(),
    )
    .unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    for (i, guest) in guests.iter_mut().enumerate() {
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

        let mut assigned_harts_count: usize = 0;
        for hart in guest.assigned_harts.iter().flatten() {
            all_harts[*hart as usize].guests.push(Some(i));
            assigned_harts_count += 1;
        }

        while guest.assigned_harts.len() < max_supported_assigned_harts_per_guest {
            guest.assigned_harts.push(None);
        }

        // Add guest entry in GUESTS array.
        writeln!(
            f,
            "Guest {{
    entry_gpa: {},
    active_hart_count: core::sync::atomic::AtomicUsize::new(0),
    active_assigned_hart_count: core::sync::atomic::AtomicUsize::new(0),
    harts_cap: {},
    assigned_harts_cap: {assigned_harts_count},
    assigned_harts: {:#?},
    data: include_bytes!(\"{manifest_dir}/{}\"),
    vcpus: core::cell::UnsafeCell::new({vcpus}),
    vcpu_ptrs: spin::Mutex::new([const {{ None }}; MAX_SUPPORTED_HARTS_PER_GUEST]),
}},

",
            guest.entry_gpa,
            guest.harts_cap - assigned_harts_count,
            guest.assigned_harts,
            paths[i]
        )
        .unwrap();
    }

    writeln!(f, "];").unwrap();

    writeln!(
        f,
        "
static HARTS: [Hart; HARTS_CAP_SUM] = [
"
    )
    .unwrap();

    for hart in all_harts {
        writeln!(
            f,
            "
Hart {{
    assigned_guests: core::cell::UnsafeCell::new([core::ptr::null_mut(); MAX_SUPPORTED_ASSIGNED_GUESTS_PER_HART]),
    guests: ["
        )
        .unwrap();
        for guest_id in &hart.guests {
            writeln!(f, "{guest_id:#?},").unwrap();
        }
        for _ in 0..(max_supported_assigned_guests_per_hart - hart.guests.len()) {
            writeln!(f, "None,").unwrap();
        }
        writeln!(f, "]}},").unwrap();
    }

    writeln!(f, "];").unwrap();
}
