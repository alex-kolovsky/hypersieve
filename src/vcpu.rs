use crate::{allocator::alloc_pages, guest_table::GuestPageTable};
use core::{arch::asm, default::Default};

#[derive(Debug, Default)]
pub struct Vcpu {
    // Host registers
    pub host_sp: u64,
    // General purpose registers
    pub ra: u64,
    pub sp: u64,
    pub gp: u64,
    pub tp: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub s0: u64,
    pub s1: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub a6: u64,
    pub a7: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub t6: u64,

    // CSRs
    pub hstatus: u64,
    pub hgatp: u64,
    pub sstatus: u64,
    pub sepc: u64,
    pub scause: u64,

    // Virtual registers
    pub vsstatus: u64,
}
impl Vcpu {
    pub fn new(table: &GuestPageTable, guest_entry: u64) -> Self {
        let mut hstatus: u64 = 0;
        hstatus |= 2 << 32; // set VSXL to 64 bits
        hstatus |= 1 << 7; // set

        let vs: u64 = 1 << 9; // VS: Vector Status (Initial)
        let vsstatus = vs;

        let spp: u64 = 1 << 8; // SPP: Supervisor Previous Privilege mode (VS-mode)
        let sstatus: u64 = spp | vs;

        let stack_size = 512 * 1024;
        let host_sp: u64 = alloc_pages(stack_size) as u64 + stack_size as u64;

        Self {
            hstatus,
            hgatp: table.hgatp(),
            sstatus,
            sepc: guest_entry,
            host_sp,
            vsstatus,
            ..Default::default()
        }
    }
    pub fn run(&mut self) -> ! {
        unsafe {
            asm!(
                "csrw hstatus, {hstatus}",
                "csrw sstatus, {sstatus}",
                "csrw sscratch, {sscratch}",
                "csrw hgatp, {hgatp}",
                "csrw sepc, {sepc}",
                "csrw vsstatus, {vsstatus}",
                "sret",
                hstatus = in(reg) self.hstatus,
                sstatus = in(reg) self.sstatus,
                hgatp = in(reg) self.hgatp,
                sepc = in(reg) self.sepc,
                sscratch = in(reg) (self as *mut Vcpu as usize),
                vsstatus = in(reg) (self.vsstatus),
            );
        }
        unreachable!();
    }
}
