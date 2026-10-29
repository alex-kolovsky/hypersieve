use crate::{GUESTS, VLEN, read_csr, registers};
use core::{arch::naked_asm, default::Default, mem::offset_of, sync::atomic::Ordering};

pub const TIMER_OFFSET: u64 = 10_000;
pub type Vr = [u32; VLEN / 32];

#[repr(C)]
#[repr(align(8))]
#[derive(Debug, Default)]
pub struct Vcpu {
    // General purpose registers
    pub gprs: [u64; 32],

    pub host_hart_id: usize,
    pub guest_id_for_hart: usize,

    // Host registers
    pub host_sp: u64,

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
    pub vrs: [Vr; 32],

    // Vector CSRs
    pub vtype: u64,
    pub vl: u64,

    // Floating-point registers
    pub fprs: [f64; 32],
}

impl Vcpu {
    pub const fn zeroed() -> Self {
        Self {
            host_hart_id: 0,
            guest_id_for_hart: 0,

            // General purpose registers
            gprs: [0; 32],

            // Host registers
            host_sp: 0,

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
            vtype: 0,
            vl: 0,

            // Vector registers
            vrs: [[0; VLEN / 32]; 32],

            // Floating-point registers
            fprs: [0.0; 32],
        }
    }
    pub fn new(guest_entry: u64, virtual_hart_id: u64) -> Self {
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

        // Set the a0 register to the virtual hart ID.
        let mut gprs = [0; 32];
        gprs[registers::A0_OFFSET] = virtual_hart_id;

        Self {
            hstatus,
            sstatus,
            sepc: guest_entry,
            host_sp,
            vsstatus,
            gprs,
            ..Default::default()
        }
    }
    pub fn run_next_guest(&mut self) {
        // Save these values before releasing this vcpu
        // to other harts to prevent data races.
        let host_sp = self.host_sp;
        let guest_id_for_hart = self.guest_id_for_hart;
        let host_hart_id = self.host_hart_id;

        let current_guest_id = crate::guest::get_current_guest_id(host_hart_id, guest_id_for_hart);
        let current_guest = &crate::GUESTS[current_guest_id];

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
            restore_extension_registers(next_guest);
            (*next_guest).vstimecmp = vstimecmp;
            (*next_guest).run();
        }
    }

    pub fn run(&mut self) -> ! {
        restore_extension_registers(self as *mut Vcpu);
        unsafe {
            switch_to_guest(self as *mut Vcpu, self.vstimecmp);
        }

        unreachable!();
    }

    pub fn very_fisrt_run(&mut self, host_hart_id: usize, guest_id_for_hart: usize) -> ! {
        // Set guest supervisor timer compare value.
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

        ra_offset = const offset_of!(Vcpu, gprs) + registers::RA_OFFSET * 8,
        sp_offset = const offset_of!(Vcpu, gprs) + registers::SP_OFFSET * 8,
        gp_offset = const offset_of!(Vcpu, gprs) + registers::GP_OFFSET * 8,
        tp_offset = const offset_of!(Vcpu, gprs) + registers::TP_OFFSET * 8,
        t0_offset = const offset_of!(Vcpu, gprs) + registers::T0_OFFSET * 8,
        t1_offset = const offset_of!(Vcpu, gprs) + registers::T1_OFFSET * 8,
        t2_offset = const offset_of!(Vcpu, gprs) + registers::T2_OFFSET * 8,
        s0_offset = const offset_of!(Vcpu, gprs) + registers::S0_OFFSET * 8,
        s1_offset = const offset_of!(Vcpu, gprs) + registers::S1_OFFSET * 8,
        a0_offset = const offset_of!(Vcpu, gprs) + registers::A0_OFFSET * 8,
        a1_offset = const offset_of!(Vcpu, gprs) + registers::A1_OFFSET * 8,
        a2_offset = const offset_of!(Vcpu, gprs) + registers::A2_OFFSET * 8,
        a3_offset = const offset_of!(Vcpu, gprs) + registers::A3_OFFSET * 8,
        a4_offset = const offset_of!(Vcpu, gprs) + registers::A4_OFFSET * 8,
        a5_offset = const offset_of!(Vcpu, gprs) + registers::A5_OFFSET * 8,
        a6_offset = const offset_of!(Vcpu, gprs) + registers::A6_OFFSET * 8,
        a7_offset = const offset_of!(Vcpu, gprs) + registers::A7_OFFSET * 8,
        s2_offset = const offset_of!(Vcpu, gprs) + registers::S2_OFFSET * 8,
        s3_offset = const offset_of!(Vcpu, gprs) + registers::S3_OFFSET * 8,
        s4_offset = const offset_of!(Vcpu, gprs) + registers::S4_OFFSET * 8,
        s5_offset = const offset_of!(Vcpu, gprs) + registers::S5_OFFSET * 8,
        s6_offset = const offset_of!(Vcpu, gprs) + registers::S6_OFFSET * 8,
        s7_offset = const offset_of!(Vcpu, gprs) + registers::S7_OFFSET * 8,
        s8_offset = const offset_of!(Vcpu, gprs) + registers::S8_OFFSET * 8,
        s9_offset = const offset_of!(Vcpu, gprs) + registers::S9_OFFSET * 8,
        s10_offset = const offset_of!(Vcpu, gprs) + registers::S10_OFFSET * 8,
        s11_offset = const offset_of!(Vcpu, gprs) + registers::S11_OFFSET * 8,
        t3_offset = const offset_of!(Vcpu, gprs) + registers::T3_OFFSET * 8,
        t4_offset = const offset_of!(Vcpu, gprs) + registers::T4_OFFSET * 8,
        t5_offset = const offset_of!(Vcpu, gprs) + registers::T5_OFFSET * 8,
        t6_offset = const offset_of!(Vcpu, gprs) + registers::T6_OFFSET * 8,
    );
}

fn restore_extension_registers(vcpu_ptr: *mut Vcpu) {
    let vcpu_ref = unsafe { &(*vcpu_ptr) };
    if (vcpu_ref.vsstatus & ((1 << 9) | (1 << 10))) != 0 {
        // Restore vector registers.
        crate::restore_vrs(vcpu_ptr);
    }

    if (vcpu_ref.vsstatus & ((1 << 14) | (1 << 13))) != 0 {
        // Restore floating-point registers.
        crate::restore_fprs(vcpu_ptr);
    }
}
