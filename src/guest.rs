use hypersieve_abi::GuestInfo;

use crate::{
    GUESTS, MAX_ALIGNED_PASSTHROUGH_REGIONS_COUNT, MAX_ALLOWED_MMIO_REGIONS_COUNT,
    MAX_ALLOWED_SHARED_BUFFER_COUNT_PER_GUEST, MAX_SUPPORTED_HARTS_PER_GUEST, SHARED_BUFFERS,
    allocator::alloc_pages,
    arch::{GuestPageTableFunctions, GuestPageTableInterface, Vcpu, VcpuInterface},
};
use core::{cell::UnsafeCell, sync::atomic::Ordering};

#[derive(Debug)]
pub struct MemoryRegion {
    pub base_addr: u64,
    pub size: u64,
}

#[derive(Debug)]
pub struct MmioMemoryRegion {
    pub region: MemoryRegion,
    // The 'emulation_fn' is a pointer to the handler responsible for emulating access to this region.
    pub emulation_fn: Option<fn(crate::arch::PageFaultInfo)>,
}

pub struct Guest {
    pub info: GuestInfo,
    pub page_table: spin::Mutex<crate::arch::GuestPageTable>,
    pub entry_gpa: usize,
    pub vcpu_ptrs:
        spin::Mutex<[Option<&'static mut dyn VcpuInterface>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub vcpus: UnsafeCell<[Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub data: &'static [u8],
    pub aligned_passthrough_mmio_regions: [MmioMemoryRegion; MAX_ALIGNED_PASSTHROUGH_REGIONS_COUNT],
    pub allowed_mmio_memory_region: [MmioMemoryRegion; MAX_ALLOWED_MMIO_REGIONS_COUNT],
    pub guest_specific_trap_routine: fn(&mut Vcpu),
    pub guest_specific_ret_routine: fn(&mut Vcpu),
    pub guest_specific_switch_routine: fn(&mut Vcpu),
    pub shared_buffer_ids: [Option<usize>; MAX_ALLOWED_SHARED_BUFFER_COUNT_PER_GUEST],
}

// The Guest struct is thread-safe if we never change Option::None
// to Option::Some or vice versa after waking the harts up.
unsafe impl Sync for Guest {}
unsafe impl Send for Guest {}

pub fn allocate_guest_memory(
    guest_entry_gpa: usize,
    image: &'static [u8],
) -> crate::arch::GuestPageTable {
    // Copy guest kernel to a guest memory buffer.
    let kernel_memory = alloc_pages(image.len(), 16);
    let dst = kernel_memory;
    let src = image.as_ptr();
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, image.len());
    }

    GuestPageTableFunctions::map_kernel(guest_entry_gpa, kernel_memory)
}

pub fn claim_vcpu_for_hart_if_available(guest_id: usize) -> bool {
    // Check if a guest is free. A guest is free if the active hart count
    // is less than the hart capacity. These values do not include assigned harts
    let guest = &GUESTS[guest_id];
    let is_free = guest.info.active_hart_count.try_update(
        Ordering::SeqCst,
        Ordering::SeqCst,
        |active_hart_count| {
            if active_hart_count < guest.info.hart_capacity {
                // Increment the active harts counter.
                Some(active_hart_count + 1)
            } else {
                None
            }
        },
    );

    is_free.is_ok()
}

pub fn claim_assigned_hart_slot_if_available(guest_id: usize) -> bool {
    // Check if a guest is free. A guest is free for a assigned hart if the active assigned hart count
    // is less than the assigned hart capacity. These values do not include assigned harts
    let guest = &GUESTS[guest_id];
    let is_free = guest.info.active_assigned_hart_count.try_update(
        Ordering::SeqCst,
        Ordering::SeqCst,
        |active_assigned_hart_count| {
            if active_assigned_hart_count < guest.info.assigned_hart_capacity {
                // Increment the active assigned harts counter.
                Some(active_assigned_hart_count + 1)
            } else {
                None
            }
        },
    );

    is_free.is_ok()
}

pub fn initialize_guests() {
    // Initialize all VCPUs (including those dedicated to assigned harts)
    // and map passthrough MMIO addresses in their page tables.
    for guest in GUESTS.iter() {
        let vcpus: *mut [Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST] = guest.vcpus.get();

        let page_table = allocate_guest_memory(guest.entry_gpa, guest.data);

        for vcpu_id in 0..(guest.info.hart_capacity + guest.info.assigned_hart_capacity) {
            // Get the pointer to the first vCPU in guest.vcpus.
            let base_ptr: *mut Option<Vcpu> = vcpus as *mut Option<Vcpu>;

            // Get the vcpu pointer by its ID.
            let cur_vcpu: *mut Option<Vcpu> = unsafe { base_ptr.add(vcpu_id) };

            if let Some(vcpu_ref) = unsafe { &mut *cur_vcpu } {
                // Load the allocated vcpu instead of an empty slot into guest.vcpus[i].
                *vcpu_ref = Vcpu::new(guest.entry_gpa as u64, vcpu_id as u64);

                vcpu_ref.initialize_vcpu(&page_table, guest, vcpu_id);
            } else {
                unreachable!();
            }
        }

        *guest.page_table.lock() = page_table;
    }

    // Initialize assigned hart VCPU pointers before booting the secondary cores.
    for hart in &crate::HARTS {
        for (i, guest_id_opt) in hart.guests.iter().enumerate() {
            if let Some(guest_id) = guest_id_opt {
                let guest = &GUESTS[*guest_id];
                let mut vcpu_ptrs = guest.vcpu_ptrs.lock();
                let assigned_guest_vcpus = hart.assigned_guest_vcpus.get();
                unsafe {
                    // First values of the vcpu_ptrs array are intended for assigned harts.
                    (*assigned_guest_vcpus)[i] = vcpu_ptrs[0].take();
                    // Clear the assigned hart VCPU pointers from
                    // the Guest struct to prevent other harts from using them.
                    (*vcpu_ptrs)[0] = None;
                    (*vcpu_ptrs).rotate_left(1);
                }
            } else {
                break;
            }
        }
    }

    // Allocate and map shared buffers.
    for buffer_info in SHARED_BUFFERS {
        // Allocate the buffer.
        let buffer_memory = alloc_pages(buffer_info.region.size as usize, 4);

        let reader_guest = &GUESTS[buffer_info.reader_guest_id];
        let writer_guest = &GUESTS[buffer_info.writer_guest_id];

        let buffer_size = buffer_info.region.size;

        let buffer_page_count = buffer_size >> 12;

        // Map buffer to the reader guest.
        let reader_page_table = reader_guest.page_table.lock();
        for buffer_page_id in 0..buffer_page_count {
            let page_offset = buffer_page_id * 4096;
            reader_page_table.map(
                buffer_info.region.base_addr + page_offset,
                buffer_memory as u64 + page_offset,
                GuestPageTableFunctions::read_flag() as u64,
            );
        }
        drop(reader_page_table);

        // Map buffer to the writer guest.
        let writer_page_table = writer_guest.page_table.lock();
        for buffer_page_id in 0..buffer_page_count {
            let page_offset = buffer_page_id * 4096;
            writer_page_table.map(
                buffer_info.region.base_addr + page_offset,
                buffer_memory as u64 + page_offset,
                // Allow both read and write because it is impossible to allow only write.
                GuestPageTableFunctions::write_flag() as u64
                    | GuestPageTableFunctions::read_flag() as u64,
            );
        }
        drop(writer_page_table);
    }
}
