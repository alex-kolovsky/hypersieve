use crate::{
    MAX_SUPPORTED_DEDICATED_HARTS_PER_GUEST, MAX_SUPPORTED_HARTS_PER_GUEST,
    allocator::alloc_pages,
    guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X},
    vcpu::Vcpu,
};
use core::cell::UnsafeCell;

#[derive(Debug)]
pub struct Guest {
    pub entry_gpa: usize,
    pub vcpu_ptrs: spin::Mutex<[Option<*mut Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub vcpus: UnsafeCell<[Option<Vcpu>; MAX_SUPPORTED_HARTS_PER_GUEST]>,
    pub active_hart_count: core::sync::atomic::AtomicUsize,
    pub active_dedicated_hart_count: core::sync::atomic::AtomicUsize,
    pub harts_cap: usize,
    pub dedicated_harts_cap: usize,
    pub dedicated_harts: [Option<u32>; MAX_SUPPORTED_DEDICATED_HARTS_PER_GUEST],
    pub data: &'static [u8],
}

// The Guest struct is thread-safe if we never change Option::None to Option::Some or vice versa after waking the harts up.
unsafe impl Sync for Guest {}
unsafe impl Send for Guest {}

pub fn allocate_guest_memory(guest_entry_gpa: usize, image: &'static [u8]) -> Vcpu {
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

    Vcpu::new(&table, guest_entry_gpa as u64)
}
