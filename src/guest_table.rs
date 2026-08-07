pub const PTE_R: u64 = 1 << 1;
pub const PTE_W: u64 = 1 << 2;
pub const PTE_X: u64 = 1 << 3;

const PTE_V: u64 = 1 << 0; // Valid bit
const PTE_U: u64 = 1 << 4; // User

const PPN_SHIFT: usize = 12;
const PPN_PTE_SHIFT: usize = 10;

#[repr(transparent)]
pub struct PageEntry(u64);

impl PageEntry {
    pub fn new(paddr: u64, flags: u64) -> Self {
        let ppn = paddr >> PPN_SHIFT; // Delete page offset
        Self((ppn << PPN_PTE_SHIFT) | flags)
    }

    pub fn is_valid(&self) -> bool {
        self.0 & PTE_V != 0
    }

    pub fn paddr(&self) -> u64 {
        (self.0 >> PPN_PTE_SHIFT) << PPN_SHIFT
    }
}
