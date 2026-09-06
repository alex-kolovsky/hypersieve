pub const TIMER_OFFSET: u64 = 10_000;

use crate::{GUESTS, VLEN, read_csr};
use core::{arch::naked_asm, default::Default, mem::offset_of, sync::atomic::Ordering};

#[repr(C)]
#[repr(align(8))]
#[derive(Debug, Default)]
pub struct Vcpu {
    pub host_hart_id: usize,
    pub guest_id_for_hart: usize,
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
    pub vstimecmp: u64,

    // Vector registers
    pub v0: [u32; VLEN / 32],
    pub v1: [u32; VLEN / 32],
    pub v2: [u32; VLEN / 32],
    pub v3: [u32; VLEN / 32],
    pub v4: [u32; VLEN / 32],
    pub v5: [u32; VLEN / 32],
    pub v6: [u32; VLEN / 32],
    pub v7: [u32; VLEN / 32],
    pub v8: [u32; VLEN / 32],
    pub v9: [u32; VLEN / 32],
    pub v10: [u32; VLEN / 32],
    pub v11: [u32; VLEN / 32],
    pub v12: [u32; VLEN / 32],
    pub v13: [u32; VLEN / 32],
    pub v14: [u32; VLEN / 32],
    pub v15: [u32; VLEN / 32],
    pub v16: [u32; VLEN / 32],
    pub v17: [u32; VLEN / 32],
    pub v18: [u32; VLEN / 32],
    pub v19: [u32; VLEN / 32],
    pub v20: [u32; VLEN / 32],
    pub v21: [u32; VLEN / 32],
    pub v22: [u32; VLEN / 32],
    pub v23: [u32; VLEN / 32],
    pub v24: [u32; VLEN / 32],
    pub v25: [u32; VLEN / 32],
    pub v26: [u32; VLEN / 32],
    pub v27: [u32; VLEN / 32],
    pub v28: [u32; VLEN / 32],
    pub v29: [u32; VLEN / 32],
    pub v30: [u32; VLEN / 32],
    pub v31: [u32; VLEN / 32],

    // Floating-point registers
    pub f0: f64,
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
    pub f4: f64,
    pub f5: f64,
    pub f6: f64,
    pub f7: f64,
    pub f8: f64,
    pub f9: f64,
    pub f10: f64,
    pub f11: f64,
    pub f12: f64,
    pub f13: f64,
    pub f14: f64,
    pub f15: f64,
    pub f16: f64,
    pub f17: f64,
    pub f18: f64,
    pub f19: f64,
    pub f20: f64,
    pub f21: f64,
    pub f22: f64,
    pub f23: f64,
    pub f24: f64,
    pub f25: f64,
    pub f26: f64,
    pub f27: f64,
    pub f28: f64,
    pub f29: f64,
    pub f30: f64,
    pub f31: f64,
}

impl Vcpu {
    pub const fn zeroed() -> Self {
        Self {
            host_hart_id: 0,
            guest_id_for_hart: 0,
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
            vstimecmp: 0,

            // Vector registers
            v0: [0; VLEN / 32],
            v1: [0; VLEN / 32],
            v2: [0; VLEN / 32],
            v3: [0; VLEN / 32],
            v4: [0; VLEN / 32],
            v5: [0; VLEN / 32],
            v6: [0; VLEN / 32],
            v7: [0; VLEN / 32],
            v8: [0; VLEN / 32],
            v9: [0; VLEN / 32],
            v10: [0; VLEN / 32],
            v11: [0; VLEN / 32],
            v12: [0; VLEN / 32],
            v13: [0; VLEN / 32],
            v14: [0; VLEN / 32],
            v15: [0; VLEN / 32],
            v16: [0; VLEN / 32],
            v17: [0; VLEN / 32],
            v18: [0; VLEN / 32],
            v19: [0; VLEN / 32],
            v20: [0; VLEN / 32],
            v21: [0; VLEN / 32],
            v22: [0; VLEN / 32],
            v23: [0; VLEN / 32],
            v24: [0; VLEN / 32],
            v25: [0; VLEN / 32],
            v26: [0; VLEN / 32],
            v27: [0; VLEN / 32],
            v28: [0; VLEN / 32],
            v29: [0; VLEN / 32],
            v30: [0; VLEN / 32],
            v31: [0; VLEN / 32],

            // Floating-point registers
            f0: 0.0,
            f1: 0.0,
            f2: 0.0,
            f3: 0.0,
            f4: 0.0,
            f5: 0.0,
            f6: 0.0,
            f7: 0.0,
            f8: 0.0,
            f9: 0.0,
            f10: 0.0,
            f11: 0.0,
            f12: 0.0,
            f13: 0.0,
            f14: 0.0,
            f15: 0.0,
            f16: 0.0,
            f17: 0.0,
            f18: 0.0,
            f19: 0.0,
            f20: 0.0,
            f21: 0.0,
            f22: 0.0,
            f23: 0.0,
            f24: 0.0,
            f25: 0.0,
            f26: 0.0,
            f27: 0.0,
            f28: 0.0,
            f29: 0.0,
            f30: 0.0,
            f31: 0.0,
        }
    }
    pub fn new(
        table: &crate::guest_table::GuestPageTable,
        guest_entry: u64,
        virtual_hart_id: u64,
    ) -> Self {
        // Set the XLEN for VS-mode (VSXL bitfield) to 64 bits in hstatus.
        let hstatus_vsxl: u64 = 2 << 32;
        // Set Supervisor Previous Virtualization (SPV) to 1 so the sret instruction boots the CPU into virtual mode.
        let hstatus_spv: u64 = 1 << 7;
        let hstatus: u64 = hstatus_spv | hstatus_vsxl; // hstatus: Hypervisor Status

        // Set the Vector Status (VS) and the Floating-point Status (FS)
        // bitfield in vsstatus to enable vector extension in VS-mode.
        let sstatus_vs: u64 = 1 << 9;
        let sstatus_fs = 1 << 13;

        let vsstatus = sstatus_vs | sstatus_fs; // vsstatus: Virtual Supervisor Status

        // Set Supervisor Previous Privilege mode (SPP) to 1 so the sret instruction boots the CPU into Supervisor mode (VS-mode in our case).
        let sstatus_spp: u64 = 1 << 8;
        let sstatus: u64 = sstatus_spp | sstatus_vs | sstatus_fs; // sstatus: Supervisor Status

        // Set harts stack size.
        let stack_size = 512 * 1024;
        let host_sp: u64 = crate::allocator::alloc_pages(stack_size) as u64 + stack_size as u64;

        Self {
            hstatus,
            hgatp: table.hgatp(),
            sstatus,
            sepc: guest_entry,
            host_sp,
            vsstatus,
            a0: virtual_hart_id,
            ..Default::default()
        }
    }
    pub fn run_next_guest(&mut self) {
        // Save these values before releasing this vcpu
        // to other harts to prevent data races.
        let host_sp = self.host_sp;
        let guest_id_for_hart = self.guest_id_for_hart;
        let host_hart_id = self.host_hart_id;

        let current_guest = crate::guest::get_current_guest(&host_hart_id, &guest_id_for_hart);

        let is_assigned = crate::multihart::is_hart_assigned(host_hart_id);
        let mut guest_id = guest_id_for_hart;

        if is_assigned {
            // Update the global active assigned hart counter.
            current_guest
                .active_assigned_hart_count
                .fetch_sub(1, Ordering::Acquire);
        } else {
            // If a hart is not assigned, release this vcpu.
            let mut vcpu_ptrs = current_guest.vcpu_ptrs.lock();
            for vcpu_ptr in (*vcpu_ptrs).iter_mut() {
                if vcpu_ptr.is_none() {
                    *vcpu_ptr = Some(self as *mut Vcpu);
                    break;
                }
            }

            // Update the global active hart counter.
            GUESTS[guest_id]
                .active_hart_count
                .fetch_sub(1, Ordering::Acquire);
        }

        // Increment the current guest ID to start from the next guest immediately.
        guest_id += 1;

        let next_guest: *mut Vcpu;

        loop {
            let vcpu_ptr: Result<*mut Vcpu, ()>;
            if is_assigned {
                let hart = &crate::HARTS[self.host_hart_id];
                if hart.guests.len() <= guest_id {
                    guest_id = 0;
                }
                if crate::guest::claim_assigned_hart_slot_if_available(
                    hart.guests[guest_id].unwrap(),
                )
                .is_ok()
                {
                    unsafe {
                        vcpu_ptr = Ok((*hart.assigned_guests.get())[guest_id]);
                    }
                } else {
                    vcpu_ptr = Err(());
                }
            } else {
                if GUESTS.len() <= guest_id {
                    guest_id = 0;
                }
                vcpu_ptr = crate::guest::claim_vcpu_for_hart_if_available(guest_id);
            }

            // Run a guest if it is free; otherwise, continue searching for a free guest.
            if let Ok(vcpu_ptr) = vcpu_ptr {
                next_guest = vcpu_ptr;
                // Fill the new vcpu with correct values for this hart.
                unsafe {
                    (*next_guest).guest_id_for_hart = guest_id;
                    (*next_guest).host_hart_id = host_hart_id;
                    (*next_guest).host_sp = host_sp;
                }

                break;
            } else {
                guest_id += 1;
            }
        }

        let vstimecmp = read_csr!("time") + TIMER_OFFSET;

        unsafe {
            (*next_guest).vstimecmp = vstimecmp;
            (*next_guest).run();
        }
    }

    pub fn run(&mut self) -> ! {
        // Restore vector registers.
        crate::restore_vector_registers(self as *mut Vcpu);

        // Restore floating-point registers.
        crate::restore_floating_point_registers(self as *mut Vcpu);

        unsafe {
            switch_to_guest(self as *mut Vcpu, self.vstimecmp);
        }

        unreachable!();
    }
    pub fn very_fisrt_run(&mut self, host_hart_id: usize, guest_id_for_hart: usize) -> ! {
        let time = read_csr!("time");
        let vstimecmp = time + TIMER_OFFSET;

        self.guest_id_for_hart = guest_id_for_hart;
        self.host_hart_id = host_hart_id;

        unsafe {
            // Load the hart ID into a0 on the first run.
            switch_to_guest(self as *mut Vcpu, vstimecmp);
        }

        unreachable!();
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn switch_to_guest(vcpu: *mut Vcpu, vstimecmp_val: u64) {
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

        "ld a0, {a0_offset}(t0)",
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

        hstatus_offset = const offset_of!(Vcpu, hstatus),
        sstatus_offset = const offset_of!(Vcpu, sstatus),
        hgatp_offset = const offset_of!(Vcpu, hgatp),
        sepc_offset = const offset_of!(Vcpu, sepc),
        vsstatus_offset = const offset_of!(Vcpu, vsstatus),

        vstvec_offset = const offset_of!(Vcpu, vstvec),
        vsscratch_offset = const offset_of!(Vcpu, vsscratch),
        vsepc_offset = const offset_of!(Vcpu, vsepc),
        vscause_offset = const offset_of!(Vcpu, vscause),
        vstval_offset = const offset_of!(Vcpu, vstval),
        vsie_offset = const offset_of!(Vcpu, vsie),
        vsatp_offset = const offset_of!(Vcpu, vsatp),

        ra_offset = const offset_of!(Vcpu, ra),
        sp_offset = const offset_of!(Vcpu, sp),
        gp_offset = const offset_of!(Vcpu, gp),
        tp_offset = const offset_of!(Vcpu, tp),
        t0_offset = const offset_of!(Vcpu, t0),
        t1_offset = const offset_of!(Vcpu, t1),
        t2_offset = const offset_of!(Vcpu, t2),
        s0_offset = const offset_of!(Vcpu, s0),
        s1_offset = const offset_of!(Vcpu, s1),
        a0_offset = const offset_of!(Vcpu, a0),
        a1_offset = const offset_of!(Vcpu, a1),
        a2_offset = const offset_of!(Vcpu, a2),
        a3_offset = const offset_of!(Vcpu, a3),
        a4_offset = const offset_of!(Vcpu, a4),
        a5_offset = const offset_of!(Vcpu, a5),
        a6_offset = const offset_of!(Vcpu, a6),
        a7_offset = const offset_of!(Vcpu, a7),
        s2_offset = const offset_of!(Vcpu, s2),
        s3_offset = const offset_of!(Vcpu, s3),
        s4_offset = const offset_of!(Vcpu, s4),
        s5_offset = const offset_of!(Vcpu, s5),
        s6_offset = const offset_of!(Vcpu, s6),
        s7_offset = const offset_of!(Vcpu, s7),
        s8_offset = const offset_of!(Vcpu, s8),
        s9_offset = const offset_of!(Vcpu, s9),
        s10_offset = const offset_of!(Vcpu, s10),
        s11_offset = const offset_of!(Vcpu, s11),
        t3_offset = const offset_of!(Vcpu, t3),
        t4_offset = const offset_of!(Vcpu, t4),
        t5_offset = const offset_of!(Vcpu, t5),
        t6_offset = const offset_of!(Vcpu, t6),
    );
}
