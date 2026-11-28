use std::cmp::Ordering;

use fdt::node::FdtNode;

use crate::{
    Hart,
    build_modules::{
        config_dts_parser::{GuestType, SharedBuffer},
        hardware_dts_parser::MemoryRegion,
    },
};

pub fn extract_assigned_harts(
    guest: FdtNode,
    max_supported_assigned_harts_per_guest: &mut usize,
    hart_capacity: usize,
    harts: &[Hart],
) -> Vec<Option<u32>> {
    // Extract assigned harts.

    let mut guest_assigned_harts_vec: Vec<Option<u32>> = Vec::new();
    if let Some(guest_assigned_harts) = guest.property("assigned-harts") {
        // Convert a raw `assigned-harts` DTB property into native CPU Hart IDs.
        let guest_assigned_harts_raw = guest_assigned_harts.value;

        if guest_assigned_harts_raw.len() % 4 != 0 {
            panic!("[FATAL][CONFIG]: assigned-harts cell is not a multiple of 4 bytes");
        }

        let (chunks, _remainder) = guest_assigned_harts_raw.as_chunks::<4>();

        for bytes in chunks.iter() {
            let hart_id: u32 = u32::from_be_bytes(*bytes);
            guest_assigned_harts_vec.push(Some(hart_id));
            *max_supported_assigned_harts_per_guest += 1;
        }

        if harts.len() > hart_capacity {
            panic!(
                        "[FATAL][CONFIG]: assigned harts count cannot be bigger than the whole hart capacity
    assigned harts count: {}
    whole hart capacity: {hart_capacity}
    guest name: {}",
                        harts.len(),
                        guest.name,
                    );
        }

        // Find the maximum number of assigned harts among all VMs.
        if harts.len() > *max_supported_assigned_harts_per_guest {
            *max_supported_assigned_harts_per_guest = harts.len();
        }
    }
    guest_assigned_harts_vec
}

pub fn extract_allowed_to_write_shared_buffers(
    guest: FdtNode,
    guest_id: usize,
    guest_type: GuestType,
    shared_buffers: &mut [SharedBuffer],
) -> (Option<Vec<u32>>, usize) {
    let mut allowed_to_write_shared_buffer_count: usize = 0;
    let mut allowed_to_write_shared_buffers: Option<Vec<u32>> = None;
    if let Some(allowed_to_write_shared_buffers_raw) =
        guest.property("allowed-to-write-shared-buffers")
    {
        let mut allowed_to_write_shared_buffers_vec: Vec<u32> = Vec::new();

        let (chunks, _remainder) = allowed_to_write_shared_buffers_raw.value.as_chunks::<4>();

        for bytes in chunks.iter() {
            let allowed_to_write_shared_buffer_region_id: u32 = u32::from_be_bytes(*bytes);

            // Ensure that buffer is allowed to write only for one guest.
            if shared_buffers[allowed_to_write_shared_buffer_region_id as usize]
                .writer_guest_id
                .is_some()
            {
                panic!("[FATAL][CONFIG]: buffer cannot be allowed to write for more than one guest")
            }

            shared_buffers[allowed_to_write_shared_buffer_region_id as usize].writer_guest_id =
                Some(guest_id);
            shared_buffers[allowed_to_write_shared_buffer_region_id as usize].writer_guest_type =
                Some(guest_type);

            allowed_to_write_shared_buffers_vec.push(allowed_to_write_shared_buffer_region_id);
        }
        allowed_to_write_shared_buffer_count = allowed_to_write_shared_buffers_vec.len();
        allowed_to_write_shared_buffers = Some(allowed_to_write_shared_buffers_vec);
    }

    (
        allowed_to_write_shared_buffers,
        allowed_to_write_shared_buffer_count,
    )
}

pub fn extract_allowed_to_read_shared_buffers(
    guest: FdtNode,
    guest_id: usize,
    guest_type: GuestType,
    shared_buffers: &mut [SharedBuffer],
) -> (Option<Vec<u32>>, usize) {
    let mut allowed_to_read_shared_buffer_count: usize = 0;
    let mut allowed_to_read_shared_buffers = None;
    if let Some(allowed_to_read_shared_buffers_raw) =
        guest.property("allowed-to-read-shared-buffers")
    {
        let mut allowed_to_read_shared_buffers_vec: Vec<u32> = Vec::new();

        let (chunks, _remainder) = allowed_to_read_shared_buffers_raw.value.as_chunks::<4>();

        for bytes in chunks.iter() {
            let allowed_to_read_shared_buffer_region_id: u32 = u32::from_be_bytes(*bytes);

            // Ensure that buffer is allowed to read only for one guest.
            if shared_buffers[allowed_to_read_shared_buffer_region_id as usize]
                .reader_guest_id
                .is_some()
            {
                panic!("[FATAL][CONFIG]: buffer cannot be allowed to read for more than one guest")
            }

            shared_buffers[allowed_to_read_shared_buffer_region_id as usize].reader_guest_id =
                Some(guest_id);

            shared_buffers[allowed_to_read_shared_buffer_region_id as usize].reader_guest_type =
                Some(guest_type);

            allowed_to_read_shared_buffers_vec.push(allowed_to_read_shared_buffer_region_id);
        }

        allowed_to_read_shared_buffer_count = allowed_to_read_shared_buffers_vec.len();
        allowed_to_read_shared_buffers = Some(allowed_to_read_shared_buffers_vec);
    }
    (
        allowed_to_read_shared_buffers,
        allowed_to_read_shared_buffer_count,
    )
}

pub fn extract_allowed_mmio(
    guest: FdtNode,
    system_mmio_windows: &[MemoryRegion],
) -> Vec<MemoryRegion> {
    let allowed_mmio_raw = guest
        .property("allowed-mmio")
        .expect("[FATAL][CONFIG]: no allowed-mmio field in the hardware DTS");

    let mut allowed_mmio: Vec<MemoryRegion> = Vec::new();
    let (chunks, _remainder) = allowed_mmio_raw.value.as_chunks::<8>();

    for bytes in chunks.chunks(2) {
        let base_addr: u64 = u64::from_be_bytes(bytes[0]);
        let size: u64 = u64::from_be_bytes(bytes[1]);

        // Panic if the memory address sum overflows.
        base_addr
            .checked_add(size)
            .expect("[FATAL][CONFIG]: memory address sum (base_addr + size) overflows");

        let is_in_window = system_mmio_windows
            .binary_search_by(|region| {
                if base_addr < region.base_addr {
                    Ordering::Greater
                } else if base_addr > region.end_addr() {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .is_ok();

        if is_in_window {
            allowed_mmio.push(MemoryRegion { base_addr, size });
        } else {
            panic!(
                "[FATAL][CONFIG]: allowed MMIO address {base_addr:#x} falls outside of system MMIO windows"
            );
        }
    }
    allowed_mmio
}
