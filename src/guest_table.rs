pub const PTE_R: u64 = 1 << 1;
pub const PTE_W: u64 = 1 << 2;
pub const PTE_X: u64 = 1 << 3;

const PTE_V: u64 = 1 << 0; // Valid bit
const PTE_U: u64 = 1 << 4; // User

const PPN_SHIFT: usize = 12;
const PPN_PTE_SHIFT: usize = 10;

#[repr(transparent)]
struct PageEntry(u64);

impl PageEntry {
    pub fn new(paddr: u64, flags: u64) -> Self {
        let ppn = paddr >> PPN_SHIFT; // Delete page offset.
        Self((ppn << PPN_PTE_SHIFT) | flags)
    }

    pub fn is_valid(&self) -> bool {
        self.0 & PTE_V != 0
    }

    pub fn paddr(&self) -> u64 {
        (self.0 >> PPN_PTE_SHIFT) << PPN_SHIFT
    }
}

#[repr(transparent)]
struct Table([PageEntry; 512]);

impl Table {
    pub fn alloc() -> *mut Table {
        crate::allocator::alloc_pages(size_of::<Table>()) as *mut Table
    }

    pub fn entry_by_addr(&mut self, guest_paddr: u64, level: usize) -> &mut PageEntry {
        // Page entry contains a 12-bit offset that we shift, and it's also divided into blocks of 9 bits.
        // Each block is represented by the page table `level`.
        let index = (guest_paddr >> (12 + 9 * level)) & 0x1ff;
        &mut self.0[index as usize]
    }
}
