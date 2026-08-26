use crate::MAX_HARTS_CAP;
use crate::allocator::alloc_pages;
use crate::guest_table::{GuestPageTable, PTE_R, PTE_W, PTE_X};

#[derive(Debug)]
pub struct Guest {
    pub entry_gpa: usize,
    pub vcpu_ptrs: core::cell::UnsafeCell<
        [Option<core::sync::atomic::AtomicPtr<crate::vcpu::Vcpu>>; MAX_HARTS_CAP],
    >,
    pub vcpus: core::cell::UnsafeCell<[Option<crate::vcpu::Vcpu>; crate::MAX_HARTS_CAP]>,
    pub harts: core::sync::atomic::AtomicUsize,
    pub harts_cap: usize,
    pub data: &'static [u8],
}

// The Guest struct is thread-safe if we never change Option::None to Option::Some or vice versa after waking the harts up.
unsafe impl Sync for Guest {}
unsafe impl Send for Guest {}

pub fn allocate_guest_memory(guest_entry_gpa: usize, image: &'static [u8]) -> crate::vcpu::Vcpu {
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

    crate::vcpu::Vcpu::new(&table, guest_entry_gpa as u64)
}
