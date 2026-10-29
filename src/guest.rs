use crate::{
    GUESTS, MAX_ALIGNED_PASSTHROUGH_REGIONS_COUNT, MAX_ALLOWED_MMIO_REGIONS_COUNT,
    MAX_SUPPORTED_ASSIGNED_HARTS_PER_GUEST, MAX_SUPPORTED_HARTS_PER_GUEST,
    allocator::alloc_pages,
    guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X},
    vcpu::Vcpu,
};
use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct MmioMemoryRegion {
    pub base_addr: u64,
    pub size: u64,

    // The 'emulation_fn' is a pointer to the handler responsible for emulating access to this region.
    pub emulation_fn: Option<fn(crate::trap::page_fault::PageFaultInfo)>,
}

#[derive(Debug)]
pub struct Guest {
    pub page_table: spin::Mutex<GuestPageTable>,
    pub entry_gpa: usize,
    pub vcpu_ptrs: spin::Mutex<[Option<*mut Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub vcpus: UnsafeCell<[Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub active_hart_count: AtomicUsize,
    pub active_assigned_hart_count: AtomicUsize,
    pub hart_capacity: usize,
    pub assigned_hart_capacity: usize,
    pub assigned_harts: [Option<u32>; MAX_SUPPORTED_ASSIGNED_HARTS_PER_GUEST],
    pub data: &'static [u8],
    pub aligned_passthrough_mmio_regions: [MmioMemoryRegion; MAX_ALIGNED_PASSTHROUGH_REGIONS_COUNT],
    pub allowed_mmio_memory_region: [MmioMemoryRegion; MAX_ALLOWED_MMIO_REGIONS_COUNT],
}

// The Guest struct is thread-safe if we never change Option::None to Option::Some or vice versa after waking the harts up.
unsafe impl Sync for Guest {}
unsafe impl Send for Guest {}

pub fn allocate_guest_memory(
    guest_entry_gpa: usize,
    image: &'static [u8],
    virtual_hart_id: u64,
) -> (Vcpu, GuestPageTable) {
    // Copy guest kernel to a guest memory buffer.
    let kernel_memory = alloc_pages(image.len());
    let dst = kernel_memory;
    let src = image.as_ptr();
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, image.len());
    }

    // Map the guest memory into the guest page table.
    let table = GuestPageTable::new();
    table.map(
        guest_entry_gpa as u64,
        kernel_memory as u64,
        PTE_R | PTE_W | PTE_X,
    );

    (Vcpu::new(guest_entry_gpa as u64, virtual_hart_id), table)
}

pub fn claim_vcpu_for_hart_if_available(guest_id: usize) -> Result<*mut Vcpu, ()> {
    // Check if a guest is free. A guest is free if the active hart count
    // is less than the hart capacity. These values do not include assigned harts
    let guest = &GUESTS[guest_id];
    let mut vcpu_ptrs = guest.vcpu_ptrs.lock();
    let is_free = guest.active_hart_count.try_update(
        Ordering::SeqCst,
        Ordering::SeqCst,
        |active_hart_count| {
            if active_hart_count < guest.hart_capacity {
                // Increment the active harts counter.
                Some(active_hart_count + 1)
            } else {
                None
            }
        },
    );

    if is_free.is_ok() {
        // Remove the claimed vcpu pointer of a hart from the vcpu_ptrs array.
        let vcpu_ptr = vcpu_ptrs[0].unwrap();
        vcpu_ptrs[0] = None;
        vcpu_ptrs.rotate_left(1);

        // Drop the mutex lock.
        drop(vcpu_ptrs);
        Ok(vcpu_ptr)
    } else {
        Err(())
    }
}

pub fn claim_assigned_hart_slot_if_available(guest_id: usize) -> Result<(), ()> {
    // Check if a guest is free. A guest is free for a assigned hart if the active assigned hart count
    // is less than the assigned hart capacity. These values do not include assigned harts
    let guest = &GUESTS[guest_id];
    let is_free = guest.active_assigned_hart_count.try_update(
        Ordering::SeqCst,
        Ordering::SeqCst,
        |active_assigned_hart_count| {
            if active_assigned_hart_count < guest.assigned_hart_capacity {
                // Increment the active assigned harts counter.
                Some(active_assigned_hart_count + 1)
            } else {
                None
            }
        },
    );

    if is_free.is_ok() { Ok(()) } else { Err(()) }
}

pub fn initialize_guests() {
    // Initialize all VCPUs (including those dedicated to assigned harts)
    // and map passthrough MMIO addresses in their page tables.
    for (guest_id, guest) in GUESTS.iter().enumerate() {
        let vcpus: *mut [Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST] = guest.vcpus.get();

        for i in 0..(guest.hart_capacity + guest.assigned_hart_capacity) {
            // Get the pointer to the first vCPU in guest.vcpus.
            let base_ptr: *mut Option<Vcpu> = vcpus as *mut Option<Vcpu>;

            // Get the vcpu pointer by its ID.
            let cur_vcpu: *mut Option<Vcpu> = unsafe { base_ptr.add(i) };

            unsafe {
                if let Some(vcpu_ref) = &mut *cur_vcpu {
                    let vcpu: *mut Vcpu = vcpu_ref as *mut Vcpu;

                    // Load the allocated vcpu instead of an empty slot into guest.vcpus[i].
                    let page_table;
                    (*vcpu, page_table) =
                        allocate_guest_memory(guest.entry_gpa, guest.data, i as u64);

                    // Configure hgatp to enforce the guest physical memory mapping.
                    (*vcpu).hgatp = page_table.hgatp();

                    // Initialize aligned MMIO addresses.
                    for region in &guest.aligned_passthrough_mmio_regions {
                        if region.base_addr != 0 {
                            page_table.map(region.base_addr, region.base_addr, PTE_R | PTE_W);
                        }
                    }

                    *guest.page_table.lock() = page_table;

                    // Store this VCPU pointer into the guest's VCPU pointer array (guest.vcpu_ptrs[i]).
                    let mut vcpu_ptr = guest.vcpu_ptrs.lock();
                    (*vcpu_ptr)[i] = Some(vcpu);
                } else {
                    unreachable!();
                }
            }
        }
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
                    (*assigned_guest_vcpus)[i] = vcpu_ptrs[0].unwrap();
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
}

#[inline(always)]
pub fn get_current_guest_id(hart_id: usize, guest_id_for_hart: usize) -> usize {
    // If hart is assigned, take a guest from the local array of dedicated guests.
    if crate::multihart::is_hart_assigned(hart_id) {
        let hart = &crate::HARTS[hart_id];

        hart.guests[guest_id_for_hart].unwrap()

    // If hart is not assigned, take a guest from the global guests array.
    } else {
        guest_id_for_hart
    }
}
