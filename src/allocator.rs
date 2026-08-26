use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
};
use spin::Mutex;

unsafe extern "C" {
    pub static __heap_start: u8;
    pub static __heap_end: u8;
}
extern crate alloc;

pub const HEAP_MAX_CAPACITY: usize = 30; // max capacity = 2^(HEAP_MAX_CAPACITY)

unsafe impl<const ORDER: usize> Send for BuddyAllocator<ORDER> {}
unsafe impl<const ORDER: usize> Sync for BuddyAllocator<ORDER> {}

#[global_allocator]
pub static BUDDY_ALLOCATOR: BuddyAllocator<HEAP_MAX_CAPACITY> = BuddyAllocator {
    heap: Mutex::new(Heap::empty()),
};

pub const FREE_LIST_LENGTH: usize = HEAP_MAX_CAPACITY;

pub struct BuddyAllocator<const ORDER: usize> {
    pub heap: Mutex<Heap<ORDER>>,
}
impl<const ORDER: usize> BuddyAllocator<ORDER> {}

pub struct Heap<const ORDER: usize> {
    pub free_list: [*mut u8; ORDER],
    pub base_addr: usize,
}

impl<const ORDER: usize> Heap<ORDER> {
    pub const fn empty() -> Self {
        Self {
            free_list: [null_mut(); ORDER],
            base_addr: 0x0,
        }
    }

    pub fn init(&mut self) {
        let heap_start: usize = unsafe { &__heap_start as *const u8 as usize };
        let heap_end: usize = unsafe { &__heap_end as *const u8 as usize };
        self.base_addr = heap_start;
        let size_of_heap = heap_end - heap_start;

        let mut heap_node_index: Option<usize> = None;
        // Find address for first full-heap block.
        for node_index in 0..=self.free_list.len() {
            if (1 << (node_index + 1)) == size_of_heap {
                heap_node_index = Some(node_index);
                break;
            }
        }
        self.free_list[heap_node_index
            .expect("Heap size must be a power of two or less than the maximum limit ( 2^HEAP_MAX_CAPACITY )")] =
            heap_start as *mut u8;
    }

    pub fn allocate(&mut self, mut size: usize) -> *mut u8 {
        // Minimal size is 8.
        if size < 8 {
            size = 8;
        }
        let mut best_fit_node: *mut u8 = null_mut();
        let best_fit_id = size.next_power_of_two().trailing_zeros() as usize;

        for found_node_index in best_fit_id..FREE_LIST_LENGTH {
            if (1usize << found_node_index) >= size {
                let mut node = self.free_list[found_node_index];

                if !node.is_null() {
                    let splits = found_node_index - best_fit_id;
                    if splits > 0 {
                        self.free_list[found_node_index] = null_mut();
                        for index in 1..=splits {
                            let size_of_node = (1 << found_node_index) >> (index);
                            let (next_node, second_addr) = split_node(node as usize, size_of_node);
                            let buddy_ptr = second_addr as *mut u8;
                            let same_size_ptr = self.free_list[found_node_index - index];
                            if same_size_ptr.is_null() {
                                self.free_list[found_node_index - index] = buddy_ptr;
                            } else {
                                unsafe {
                                    let target = buddy_ptr as *mut *mut u8;
                                    *target = null_mut();
                                }
                            }

                            node = next_node as *mut u8;
                        }
                        best_fit_node = node;
                        break;
                    } else {
                        let ptr = node as *mut *mut u8;
                        self.free_list[found_node_index] = unsafe { *ptr };
                        best_fit_node = node;
                        return best_fit_node;
                    }
                }
            }
        }

        if best_fit_node.is_null() {
            panic!("Out of memory");
        }
        best_fit_node
    }
}

fn split_node(start_addr: usize, size: usize) -> (usize, usize) {
    let half_size = size;

    let first_addr = start_addr;
    let second_addr = first_addr + half_size;

    (first_addr, second_addr)
}

unsafe impl<const ORDER: usize> GlobalAlloc for BuddyAllocator<ORDER> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.heap.lock();

        heap.allocate(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let buddy_size = layout.size().next_power_of_two();
        let mut best_fit_size: usize = if buddy_size < 8 { 8 } else { buddy_size };
        let mut best_fit_id: usize = best_fit_size.trailing_zeros() as usize;

        let heap_start = unsafe { &__heap_start as *const u8 } as usize;
        let mut heap = self.heap.lock();

        let mut start_addr_offset = (ptr as usize) - heap_start;

        loop {
            let same_size_node = heap.free_list[best_fit_id];

            if !same_size_node.is_null() {
                let buddy_addr = (start_addr_offset ^ best_fit_size) + heap_start;

                if same_size_node as usize == buddy_addr {
                    heap.free_list[best_fit_id] = unsafe { (*same_size_node) as *mut u8 };
                } else if delete_buddy_from_linked_list(same_size_node, buddy_addr).is_err() {
                    break;
                }

                start_addr_offset = if (same_size_node as usize) > (start_addr_offset + heap_start)
                {
                    start_addr_offset
                } else {
                    (same_size_node as usize) - heap_start
                };
                best_fit_size *= 2;
                best_fit_id += 1;
            } else {
                heap.free_list[best_fit_id] = (start_addr_offset + heap_start) as *mut u8;
                break;
            }
        }
    }
}

fn delete_buddy_from_linked_list(
    mut linked_list_start: *mut u8,
    buddy_addr: usize,
) -> Result<(), ()> {
    loop {
        if linked_list_start.is_null() {
            // List ended.
            return Err(());
        } else if linked_list_start as usize == buddy_addr {
            unsafe {
                let next_node = *(*linked_list_start as *mut u8);
                *linked_list_start = next_node;
            }
            return Ok(());
        } else {
            unsafe {
                linked_list_start = *linked_list_start as *mut u8;
            }
        }
    }
}

pub fn alloc_pages(len: usize) -> *mut u8 {
    let aligned_len = (len + 4095) & !4095;
    let layout = Layout::from_size_align(aligned_len, 4096).expect("Invalid layout parameters");
    unsafe { BUDDY_ALLOCATOR.alloc_zeroed(layout) }
}
