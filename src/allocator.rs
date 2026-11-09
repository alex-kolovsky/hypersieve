use core::alloc::GlobalAlloc;

use buddy_system_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

unsafe extern "C" {
    pub static __heap_start: u8;
    pub static __heap_end: u8;
}

pub fn alloc_pages(len: usize) -> *mut u8 {
    let aligned_len = (len + 4095) & !4095;
    let layout =
        core::alloc::Layout::from_size_align(aligned_len, 4096).expect("Invalid layout parameters");
    unsafe { ALLOCATOR.alloc_zeroed(layout) }
}

pub fn init_heap() {
    let heap_start: usize = (&raw const __heap_start) as usize;
    let heap_end: usize = (&raw const __heap_end) as usize;
    let heap_size: usize = heap_end - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
