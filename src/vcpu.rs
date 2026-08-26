use crate::{allocator::alloc_pages, guest_table::GuestPageTable, read_csr};
use core::default::Default;

#[repr(C)]
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

    // Virtual-mode registers
    pub vsstatus: u64,
    pub vstvec: u64,
    pub vsscratch: u64,
    pub vsepc: u64,
    pub vscause: u64,
    pub vstval: u64,
    pub vsie: u64,
    pub vsatp: u64,
}
impl Vcpu {
    pub const fn zeroed() -> Self {
        Self {
            // Host registers
            host_sp: 0,
            // General purpose registers
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            s0: 0,
            s1: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,

            // CSRs
            hstatus: 0,
            hgatp: 0,
            sstatus: 0,
            sepc: 0,
            scause: 0,

            // Virtual-mode registers
            vsstatus: 0,
            vstvec: 0,
            vsscratch: 0,
            vsepc: 0,
            vscause: 0,
            vstval: 0,
            vsie: 0,
            vsatp: 0,
        }
    }
    pub fn new(table: &GuestPageTable, guest_entry: u64) -> Self {
        // Set the XLEN for VS-mode (VSXL bitfield) to 64 bits in hstatus.
        let vsxl: u64 = 2 << 32;
        // Set Supervisor Previous Virtualization (SPV) to 1 so the sret instruction boots the CPU into virtual mode.
        let spv: u64 = 1 << 7;
        let hstatus: u64 = spv | vsxl; // hstatus: Hypervisor Status

        // Set the Vector Status (VS) bitfield in vsstatus to enable vector extension in VS-mode.
        let vs: u64 = 1 << 9;
        let vsstatus = vs; // vsstatus: Virtual Supervisor Status

        // Set Supervisor Previous Privilege mode (SPP) to 1 so the sret instruction boots the CPU into Supervisor mode (VS-mode in our case).
        let spp: u64 = 1 << 8;
        let sstatus: u64 = spp | vs; // sstatus: Supervisor Status

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
        let time = read_csr!("time");
        let vstimecmp = time + 10_000;

        unsafe {
            switch_to_guest(self as *mut Vcpu, vstimecmp as usize, self.a0);
        }

        unreachable!();
    }
    pub fn very_fisrt_run(&mut self, hart_id: usize) -> ! {
        let time = read_csr!("time");
        let vstimecmp = time + 10_000;

        unsafe {
            // Load the hart ID into a0 on the first run.
            switch_to_guest(self as *mut Vcpu, vstimecmp as usize, hart_id as u64);
        }

        unreachable!();
    }
}

use core::arch::naked_asm;

#[unsafe(naked)]
pub unsafe extern "C" fn switch_to_guest(vcpu: *mut Vcpu, vstimecmp_val: usize, a0: u64) {
    naked_asm!(
        "mv t0, a0",

        "ld t1, {hstatus_offset}(t0)",
        "csrw hstatus, t1",

        "ld t1, {sstatus_offset}(t0)",
        "csrw sstatus, t1",

        "ld t1, {hgatp_offset}(t0)",
        "csrw hgatp, t1",

        "ld t1, {sepc_offset}(t0)",
        "csrw sepc, t1",

        "ld t1, {vsstatus_offset}(t0)",
        "csrw vsstatus, t1",

        "ld t1, {vstvec_offset}(t0)",
        "csrw vstvec, t1",

        "ld t1, {vsscratch_offset}(t0)",
        "csrw vsscratch, t1",

        "ld t1, {vsepc_offset}(t0)",
        "csrw vsepc, t1",

        "ld t1, {vscause_offset}(t0)",
        "csrw vscause, t1",

        "ld t1, {vstval_offset}(t0)",
        "csrw vstval, t1",

        "ld t1, {vsie_offset}(t0)",
        "csrw vsie, t1",

        "ld t1, {vsatp_offset}(t0)",
        "csrw vsatp, t1",

        "csrw vstimecmp, a1",

        "csrw sscratch, t0",

        "ld ra, {ra_offset}(t0)",
        "ld sp, {sp_offset}(t0)",
        "ld gp, {gp_offset}(t0)",
        "ld tp, {tp_offset}(t0)",
        "ld t1, {t1_offset}(t0)",
        "ld t2, {t2_offset}(t0)",
        "ld s0, {s0_offset}(t0)",
        "ld s1, {s1_offset}(t0)",

        "mv a0, a2",
        "ld a1, {a1_offset}(t0)",
        "ld a2, {a2_offset}(t0)",
        "ld a3, {a3_offset}(t0)",
        "ld a4, {a4_offset}(t0)",
        "ld a5, {a5_offset}(t0)",
        "ld a6, {a6_offset}(t0)",
        "ld a7, {a7_offset}(t0)",

        "ld s2, {s2_offset}(t0)",
        "ld s3, {s3_offset}(t0)",
        "ld s4, {s4_offset}(t0)",
        "ld s5, {s5_offset}(t0)",
        "ld s6, {s6_offset}(t0)",
        "ld s7, {s7_offset}(t0)",
        "ld s8, {s8_offset}(t0)",
        "ld s9, {s9_offset}(t0)",
        "ld s10, {s10_offset}(t0)",
        "ld s11, {s11_offset}(t0)",

        "ld t3, {t3_offset}(t0)",
        "ld t4, {t4_offset}(t0)",
        "ld t5, {t5_offset}(t0)",
        "ld t6, {t6_offset}(t0)",

        "ld t0, {t0_offset}(t0)",

        "sret",

        hstatus_offset = const core::mem::offset_of!(Vcpu, hstatus),
        sstatus_offset = const core::mem::offset_of!(Vcpu, sstatus),
        hgatp_offset = const core::mem::offset_of!(Vcpu, hgatp),
        sepc_offset = const core::mem::offset_of!(Vcpu, sepc),
        vsstatus_offset = const core::mem::offset_of!(Vcpu, vsstatus),

        vstvec_offset = const core::mem::offset_of!(Vcpu, vstvec),
        vsscratch_offset = const core::mem::offset_of!(Vcpu, vsscratch),
        vsepc_offset = const core::mem::offset_of!(Vcpu, vsepc),
        vscause_offset = const core::mem::offset_of!(Vcpu, vscause),
        vstval_offset = const core::mem::offset_of!(Vcpu, vstval),
        vsie_offset = const core::mem::offset_of!(Vcpu, vsie),
        vsatp_offset = const core::mem::offset_of!(Vcpu, vsatp),

        ra_offset = const core::mem::offset_of!(Vcpu, ra),
        sp_offset = const core::mem::offset_of!(Vcpu, sp),
        gp_offset = const core::mem::offset_of!(Vcpu, gp),
        tp_offset = const core::mem::offset_of!(Vcpu, tp),
        t0_offset = const core::mem::offset_of!(Vcpu, t0),
        t1_offset = const core::mem::offset_of!(Vcpu, t1),
        t2_offset = const core::mem::offset_of!(Vcpu, t2),
        s0_offset = const core::mem::offset_of!(Vcpu, s0),
        s1_offset = const core::mem::offset_of!(Vcpu, s1),
        a1_offset = const core::mem::offset_of!(Vcpu, a1),
        a2_offset = const core::mem::offset_of!(Vcpu, a2),
        a3_offset = const core::mem::offset_of!(Vcpu, a3),
        a4_offset = const core::mem::offset_of!(Vcpu, a4),
        a5_offset = const core::mem::offset_of!(Vcpu, a5),
        a6_offset = const core::mem::offset_of!(Vcpu, a6),
        a7_offset = const core::mem::offset_of!(Vcpu, a7),
        s2_offset = const core::mem::offset_of!(Vcpu, s2),
        s3_offset = const core::mem::offset_of!(Vcpu, s3),
        s4_offset = const core::mem::offset_of!(Vcpu, s4),
        s5_offset = const core::mem::offset_of!(Vcpu, s5),
        s6_offset = const core::mem::offset_of!(Vcpu, s6),
        s7_offset = const core::mem::offset_of!(Vcpu, s7),
        s8_offset = const core::mem::offset_of!(Vcpu, s8),
        s9_offset = const core::mem::offset_of!(Vcpu, s9),
        s10_offset = const core::mem::offset_of!(Vcpu, s10),
        s11_offset = const core::mem::offset_of!(Vcpu, s11),
        t3_offset = const core::mem::offset_of!(Vcpu, t3),
        t4_offset = const core::mem::offset_of!(Vcpu, t4),
        t5_offset = const core::mem::offset_of!(Vcpu, t5),
        t6_offset = const core::mem::offset_of!(Vcpu, t6),
    );
}
