mod extractor_functions;
mod mmio_partition;

use crate::CONFIG_DTS_PATH;
use crate::Guest;
use crate::Hart;
use crate::TargetArch;
use crate::build_modules::arch::TargetArchFeatures;
use crate::build_modules::arch::TargetArchGuestFeatures;
use crate::build_modules::config_dts_parser::extractor_functions::extract_allowed_mmio;
use crate::build_modules::config_dts_parser::extractor_functions::extract_allowed_to_read_shared_buffers;
use crate::build_modules::config_dts_parser::extractor_functions::extract_allowed_to_write_shared_buffers;
use crate::build_modules::config_dts_parser::extractor_functions::extract_assigned_harts;
use crate::build_modules::config_dts_parser::mmio_partition::partition_passthrough_mmio_regions;
use crate::build_modules::hardware_dts_parser::MemoryRegion;

#[derive(Debug)]
pub struct HypervisorConfiguration {
    pub guests: Vec<Guest>,
    pub harts: Vec<Hart>,

    pub total_hart_capacity: usize,

    pub max_supported_harts_per_guest: usize,
    pub max_supported_assigned_harts_per_guest: usize,
    pub max_allowed_mmio_peripherals_count: usize,
    pub max_aligned_passthrough_mmio_regions_count: usize,
    pub max_allowed_shared_buffer_count_per_guest: usize,

    pub target_arch_features: TargetArchFeatures,

    pub system_mmio_windows: Vec<MemoryRegion>,
    pub shared_buffers: Vec<SharedBuffer>,
}

pub struct SharedBuffer {
    pub region: MemoryRegion,
    pub reader_guest_id: Option<usize>,
    pub writer_guest_id: Option<usize>,
}

// Custom Debug implementation to print guest IDs without the Option object.
impl core::fmt::Debug for SharedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedBuffer")
            .field("region", &format_args!("{:#?}", self.region))
            .field(
                "writer_guest_id",
                &format_args!(
                    "{}",
                    self.writer_guest_id
                        .expect("[FATAL][CONFIG]: shared buffer cannot be own by only one guest")
                ),
            )
            .field(
                "reader_guest_id",
                &format_args!(
                    "{}",
                    self.reader_guest_id
                        .expect("[FATAL][CONFIG]: shared buffer cannot be own by only one guest")
                ),
            )
            .finish()
    }
}

pub fn compile_dts(dtb_output_path: &std::path::PathBuf, dts_source_path: &str) {
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
        .expect("[FATAL][BUILD]: failed to execute 'dtc'. Is device-tree-compiler installed on your system?");

    if !dtc_output.status.success() {
        panic!(
            "[FATAL][BUILD]: DTC Compilation Error: {}",
            String::from_utf8_lossy(&dtc_output.stderr)
        );
    }
}

pub fn parse_dts(
    out_dir: &str,
    target_arch: &TargetArch,
    mmio_peripherals: Vec<crate::build_modules::hardware_dts_parser::MmioPeripheral>,
) -> HypervisorConfiguration {
    let dtb_output_path = std::path::PathBuf::from(out_dir).join("config.dtb");

    // Compile Device Tree file.
    compile_dts(&dtb_output_path, CONFIG_DTS_PATH);

    // Parse Device Tree Binary file.
    let dtb_data = std::fs::read(dtb_output_path)
        .expect("[FATAL][BUILD]: failed to read the main description file");

    // A vector of all assigned harts in a system.
    let mut harts: Vec<Hart> = Vec::new();

    let mut max_supported_harts_per_guest: usize = 0;
    let mut max_supported_assigned_harts_per_guest: usize = 0;

    let mut max_allowed_mmio_peripherals_count: usize = 0;
    let mut max_aligned_passthrough_mmio_regions_count: usize = 0;
    let mut max_allowed_shared_buffer_count_per_guest: usize = 0;

    // Total hart capacity allocated across all guest VMs.
    let mut total_hart_capacity: usize = 0;

    let mut target_arch_features: TargetArchFeatures = Default::default();

    let system_mmio_windows: Vec<MemoryRegion>;

    let mut guests: Vec<Guest> = Vec::new();

    let mut shared_buffers: Vec<SharedBuffer>;

    let fdt =
        fdt::Fdt::new(&dtb_data).expect("[FATAL][BUILD]: failed to parse DT file into Fdt struct");

    if let Some(hypervisor) = fdt.find_node("/hypervisor-config") {
        let system_mmio_windows_raw = hypervisor
            .property("mmio-windows")
            .expect("[FATAL][CONFIG]: no mmio-windows property in DTS")
            .value;

        system_mmio_windows = MemoryRegion::vec_from_raw_array(system_mmio_windows_raw);

        let shared_buffers_raw = hypervisor
            .property("shared-buffers")
            .expect("[FATAL][CONFIG]: no shared-buffers property in DTS")
            .value;

        let regions = MemoryRegion::vec_from_raw_array(shared_buffers_raw);
        shared_buffers = regions
            .iter()
            .map(|region| {
                if region.is_aligned() {
                    SharedBuffer {
                        region: *region,
                        reader_guest_id: None,
                        writer_guest_id: None,
                    }
                } else {
                    panic!(
                        "[FATAL][CONFIG]: shared buffer region is not page-aligned: {region:#?}"
                    );
                }
            })
            .collect();
        shared_buffers.sort_by_key(|buffer| buffer.region.base_addr);

        for (guest_id, guest) in hypervisor.children().enumerate() {
            let mut target_arch_guest_features = TargetArchGuestFeatures::default();
            // Extract the VM entry address.
            let entry_gpa = guest
                .property("entry")
                .expect("[FATAL][CONFIG]: no entry field in DTS")
                .as_usize()
                .expect("[FATAL][CONFIG]: entry is not an integer");
            // Extract the hart capacity for the VM.
            let hart_capacity = guest
                .property("hart-capacity")
                .expect("[FATAL][CONFIG]: no hart-capacity field in DTS")
                .as_usize()
                .expect("[FATAL][CONFIG]: failed to parse hart-capacity field into usize");

            crate::build_modules::arch::parse_arch_related_values_from_dts(
                &guest,
                &mut target_arch_guest_features,
                target_arch,
                &mut target_arch_features,
            );

            let allowed_mmio = extract_allowed_mmio(guest, &system_mmio_windows);
            let (
                allowed_to_read_shared_buffers,
                allowed_to_read_shared_buffer_count,
                notify_policy,
            ) = extract_allowed_to_read_shared_buffers(guest, guest_id, &mut shared_buffers);

            let (allowed_to_write_shared_buffers, allowed_to_write_shared_buffer_count) =
                extract_allowed_to_write_shared_buffers(guest, guest_id, &mut shared_buffers);

            let allowed_shared_buffer_count =
                allowed_to_write_shared_buffer_count + allowed_to_read_shared_buffer_count;

            if allowed_shared_buffer_count >= max_allowed_shared_buffer_count_per_guest {
                max_allowed_shared_buffer_count_per_guest = allowed_shared_buffer_count;
            }

            let path = format!(
                "{}{}",
                crate::root_dir!(),
                guest
                    .property("path")
                    .expect("[FATAL][CONFIG]: no path field in DTS")
                    .as_str()
                    .expect("[FATAL][CONFIG]: failed to parse path field into string")
            );

            if !target_arch.is_wasm32() && !std::path::Path::exists(std::path::Path::new(&path)) {
                panic!("[FATAL][CONFIG]: guest file {path} is not found");
            }

            let (aligned_passthrough_mmio_regions, allowed_mmio_peripherals) =
                partition_passthrough_mmio_regions(&mmio_peripherals, &allowed_mmio);

            if allowed_mmio_peripherals.len() > max_allowed_mmio_peripherals_count {
                max_allowed_mmio_peripherals_count = allowed_mmio_peripherals.len();
            }

            if aligned_passthrough_mmio_regions.len() > max_aligned_passthrough_mmio_regions_count {
                max_aligned_passthrough_mmio_regions_count = aligned_passthrough_mmio_regions.len();
            }

            total_hart_capacity += hart_capacity;

            // Find the maximum hart capacity among all VMs.
            if hart_capacity > max_supported_harts_per_guest {
                max_supported_harts_per_guest = hart_capacity;
            }

            let guest_assigned_harts = extract_assigned_harts(
                guest,
                &mut max_supported_assigned_harts_per_guest,
                hart_capacity,
                &harts,
            );

            let guest_entry = Guest {
                entry_gpa,
                hart_capacity,
                assigned_harts: guest_assigned_harts,
                path,
                aligned_passthrough_mmio_regions,
                allowed_mmio_peripherals,
                allowed_to_write_shared_buffers,
                allowed_to_read_shared_buffers,
                target_arch_guest_features,
                notify_policy,
            };

            guests.push(guest_entry);
        }
    } else {
        panic!("[FATAL][CONFIG]: no hypervisor configuration in DTS");
    }

    // Populate Hart structures for the 'harts' vector.
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
        max_allowed_mmio_peripherals_count,
        max_aligned_passthrough_mmio_regions_count,
        max_allowed_shared_buffer_count_per_guest,
        harts,
        total_hart_capacity,
        target_arch_features,
        system_mmio_windows,
        shared_buffers,
    }
}
