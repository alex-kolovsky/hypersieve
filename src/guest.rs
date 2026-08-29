use crate::{
    GUESTS, MAX_SUPPORTED_DEDICATED_HARTS_PER_GUEST, MAX_SUPPORTED_HARTS_PER_GUEST,
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
    pub active_dedicated_hart_count: AtomicUsize,
    pub harts_cap: usize,
    pub dedicated_harts_cap: usize,
    pub dedicated_harts: [Option<u32>; MAX_SUPPORTED_DEDICATED_HARTS_PER_GUEST],
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

pub fn assign_vcpu_if_available(guest_id: usize) -> Result<*mut Vcpu, ()> {
    // Check if a guest is free. A guest is free if the active hart count
    // is less than the hart capacity. These values do not include dedicated harts
    let guest = &GUESTS[guest_id];
    let mut vcpu_ptrs = guest.vcpu_ptrs.lock();
    let is_free =
        guest
            .active_hart_count
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |active_harts| {
                if active_harts < guest.harts_cap {
                    // Increment the active harts counter.
                    Some(active_harts + 1)
                } else {
                    None
                }
            });

    if is_free.is_ok() {
        // Remove the assigned vcpu pointer of a hart from the vcpu_ptrs array.
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

pub fn assign_dedicated_vcpu_if_available(guest_id: usize) -> Result<(), ()> {
    // Check if a guest is free. A guest is free for a dedicated hart if the active dedicated hart count
    // is less than the dedicated hart capacity. These values do not include dedicated harts
    let guest = &GUESTS[guest_id];
    let is_free = guest.active_dedicated_hart_count.try_update(
        Ordering::SeqCst,
        Ordering::SeqCst,
        |active_harts| {
            if active_harts < guest.dedicated_harts_cap {
                // Increment the active dedicated harts counter.
                Some(active_harts + 1)
            } else {
                None
            }
        },
    );

    if is_free.is_ok() { Ok(()) } else { Err(()) }
}
