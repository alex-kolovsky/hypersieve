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

#[derive(Debug)]
pub struct GuestPageTable {
    table: *mut Table,
}

impl GuestPageTable {
    pub fn new() -> Self {
        Self {
            table: Table::alloc(),
        }
    }

    pub const fn empty() -> Self {
        Self {
            table: core::ptr::null_mut(),
        }
    }

    pub fn hgatp(&self) -> u64 {
        // Page-based 50-bit virtual addressing (Sv48x4).
        let mode_field = 9u64 << 60;

        let table_ppn = self.table as u64 >> PPN_SHIFT;

        mode_field | table_ppn
    }

    pub fn map(&self, guest_paddr: u64, host_paddr: u64, flags: u64) {
        let mut table = unsafe { &mut *self.table };

        for level in (1..=3).rev() {
            let entry = table.entry_by_addr(guest_paddr, level);
            if !entry.is_valid() {
                let new_table_ptr = Table::alloc();
                *entry = PageEntry::new(new_table_ptr as u64, PTE_V);
            }
            table = unsafe { &mut *(entry.paddr() as *mut Table) };
        }

        let entry = table.entry_by_addr(guest_paddr, 0);
        assert!(!entry.is_valid(), "already mapped");
        *entry = PageEntry::new(host_paddr, flags | PTE_V | PTE_U);
    }
}
