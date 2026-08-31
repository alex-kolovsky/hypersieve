use crate::{
    GUESTS, MAX_SUPPORTED_ASSIGNED_HARTS_PER_GUEST, MAX_SUPPORTED_HARTS_PER_GUEST,
    allocator::alloc_pages,
    guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X},
    vcpu::Vcpu,
};
use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct Guest {
    pub entry_gpa: usize,
    pub vcpu_ptrs: spin::Mutex<[Option<*mut Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub vcpus: UnsafeCell<[Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub active_hart_count: AtomicUsize,
    pub active_assigned_hart_count: AtomicUsize,
    pub hart_capacity: usize,
    pub assigned_hart_capacity: usize,
    pub assigned_harts: [Option<u32>; MAX_SUPPORTED_ASSIGNED_HARTS_PER_GUEST],
    pub data: &'static [u8],
}

// The Guest struct is thread-safe if we never change Option::None to Option::Some or vice versa after waking the harts up.
unsafe impl Sync for Guest {}
unsafe impl Send for Guest {}

pub fn allocate_guest_memory(
    guest_entry_gpa: usize,
    image: &'static [u8],
    virtual_hart_id: u64,
) -> Vcpu {
    // Copy guest kernel to a guest memory buffer.
    let kernel_memory = alloc_pages(image.len());
    unsafe {
        let dst = kernel_memory;
        let src = image.as_ptr();
        core::ptr::copy_nonoverlapping(src, dst, image.len());
    }

    // Map the guest memory into the guest page table.
    let table = GuestPageTable::new();
    table.map(
        guest_entry_gpa as u64,
        kernel_memory as u64,
        PTE_R | PTE_W | PTE_X,
    );

    Vcpu::new(&table, guest_entry_gpa as u64, virtual_hart_id)
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
    // Fill the guests with VCPUs.
    for guest in GUESTS.iter() {
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
                    *vcpu = allocate_guest_memory(guest.entry_gpa, guest.data, i as u64);

                    // Load this vcpu pointer into guest.vcpu_ptrs[i].
                    let mut vcpu_ptr = guest.vcpu_ptrs.lock();
                    (*vcpu_ptr)[i] = Some(vcpu);
                } else {
                    unreachable!();
                }
            }
        }

        // Initialize assigned hart pointers before booting the secondary cores.
        for assigned_hart_id in guest.assigned_harts.iter().flatten() {
            let assigned_guests = crate::HARTS[*assigned_hart_id as usize]
                .assigned_guests
                .get();

            let mut vcpu_ptrs = guest.vcpu_ptrs.lock();
            unsafe {
                for id in 0..(*assigned_guests).len() {
                    if (*assigned_guests)[id].is_null() {
                        // First values of the vcpu_ptrs array are intended for assigned harts.
                        (*assigned_guests)[id] = (*vcpu_ptrs)[0].unwrap();

                        // Remove the vcpu pointer of a assigned hart from the vcpu_ptrs array.
                        (*vcpu_ptrs)[0] = None;
                        (*vcpu_ptrs).rotate_left(1);
                    }
                }
            }
        }
    }
}
