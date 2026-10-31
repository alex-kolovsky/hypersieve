use crate::{
    registers::{self},
    trap::causes::{exceptions, interrupts},
    vcpu::Vcpu,
};
use core::{arch::naked_asm, mem::offset_of};

pub mod causes;
mod ecall;
mod fatal_exception;
pub mod page_fault;

const SXLEN: u8 = 64;

#[macro_export]
macro_rules! read_csr {
    ($csr_name:expr) => {{
        let csr_value: u64;
        unsafe {
            ::core::arch::asm!(concat!("csrr {}, ", $csr_name), out(reg) csr_value);
        }
        csr_value
    }};
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".stvec")]
#[unsafe(naked)]
pub extern "C" fn trap_handler() {
    naked_asm!(
        // Swap a0 and sscratch.
        "csrrw a0, sscratch, a0",

        // a0 is now a pointer to a VCpu. Save registers except a0.
        "sd ra, {ra_offset}(a0)",
        "sd sp, {sp_offset}(a0)",
        "sd gp, {gp_offset}(a0)",
        "sd tp, {tp_offset}(a0)",
        "sd t0, {t0_offset}(a0)",
        "sd t1, {t1_offset}(a0)",
        "sd t2, {t2_offset}(a0)",
        "sd s0, {s0_offset}(a0)",
        "sd s1, {s1_offset}(a0)",
        "sd a1, {a1_offset}(a0)",
        "sd a2, {a2_offset}(a0)",
        "sd a3, {a3_offset}(a0)",
        "sd a4, {a4_offset}(a0)",
        "sd a5, {a5_offset}(a0)",
        "sd a6, {a6_offset}(a0)",
        "sd a7, {a7_offset}(a0)",
        "sd s2, {s2_offset}(a0)",
        "sd s3, {s3_offset}(a0)",
        "sd s4, {s4_offset}(a0)",
        "sd s5, {s5_offset}(a0)",
        "sd s6, {s6_offset}(a0)",
        "sd s7, {s7_offset}(a0)",
        "sd s8, {s8_offset}(a0)",
        "sd s9, {s9_offset}(a0)",
        "sd s10, {s10_offset}(a0)",
        "sd s11, {s11_offset}(a0)",
        "sd t3, {t3_offset}(a0)",
        "sd t4, {t4_offset}(a0)",
        "sd t5, {t5_offset}(a0)",
        "sd t6, {t6_offset}(a0)",

        // Save CSRs
        "csrr t0, scause",
        "sd t0, {scause_offset}(a0)",

        "csrr t0, sepc",
        "sd t0, {sepc_offset}(a0)",

        "csrr t0, hstatus",
        "sd t0, {hstatus_offset}(a0)",

        "csrr t0, vsstatus",
        "sd t0, {vsstatus_offset}(a0)",

        "csrr t0, vstvec",
        "sd t0, {vstvec_offset}(a0)",

        "csrr t0, vsscratch",
        "sd t0, {vsscratch_offset}(a0)",

        "csrr t0, vsepc",
        "sd t0, {vsepc_offset}(a0)",

        "csrr t0, vscause",
        "sd t0, {vscause_offset}(a0)",

        "csrr t0, vstval",
        "sd t0, {vstval_offset}(a0)",

        "csrr t0, vsie",
        "sd t0, {vsie_offset}(a0)",

        "csrr t0, vsatp",
        "sd t0, {vsatp_offset}(a0)",

        "csrr t0, vstimecmp",
        "sd t0, {vstimecmp_offset}(a0)",

        "ld t0, {sstatus_offset}(a0)",
        "csrw sstatus, t0",

        // Restore a0 from sscratch, and save in to VCpu.
        "csrr t0, sscratch",
        "sd t0, {a0_offset}(a0)",

        // Switch to the hypervisor's stack.
        "ld sp, {host_sp_offset}(a0)",

        // a0 (first argument) is still the vcpu pointer here.
        "call {handle_trap}",
        handle_trap = sym handle_trap,
        // Host CSRs
        scause_offset = const offset_of!(Vcpu, scause),
        sepc_offset = const offset_of!(Vcpu, sepc),
        sstatus_offset = const offset_of!(Vcpu, sstatus),
        hstatus_offset = const offset_of!(Vcpu, hstatus),

        // Guest CSRs
        vsstatus_offset = const offset_of!(Vcpu, vsstatus),
        vstvec_offset = const offset_of!(Vcpu, vstvec),
        vsscratch_offset = const offset_of!(Vcpu, vsscratch),
        vsepc_offset = const offset_of!(Vcpu, vsepc),
        vscause_offset = const offset_of!(Vcpu, vscause),
        vstval_offset = const offset_of!(Vcpu, vstval),
        vsie_offset = const offset_of!(Vcpu, vsie),
        vsatp_offset = const offset_of!(Vcpu, vsatp),
        vstimecmp_offset = const offset_of!(Vcpu, vstimecmp),

        // Guest GPRs
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

        // Host GPRs
        host_sp_offset = const offset_of!(Vcpu, host_sp),
    );
}

pub fn handle_trap(vcpu: *mut Vcpu) {
    let vcpu_ref = unsafe { &mut (*vcpu) };

    // Execute the guest-specific trap routine, such as saving extension registers.
    let guest = unsafe { &*vcpu_ref.guest };
    let guest_specific_trap_routine = guest.guest_specific_trap_routine;
    guest_specific_trap_routine(vcpu);

    let scause = vcpu_ref.scause;

    // Extract type of trap (exception/interrupt).
    let interrupt_bit = scause >> (SXLEN - 1);
    let exception_code: u64 = scause & !(1 << 63);

    match interrupt_bit {
        // Synchronous exception
        causes::EXCEPTION => match exception_code {
            // Ecall
            exceptions::ENV_CALL_FROM_VS_MODE => ecall::handle_ecall(vcpu_ref),

            // Load guest-page fault or Store/AMO guest-page fault
            exceptions::LOAD_GUEST_PAGE_FAULT | exceptions::STORE_GUEST_PAGE_FAULT => {
                page_fault::handle_load_store_page_fault(vcpu_ref, exception_code)
            }

            _ => fatal_exception::fatal_exception(
                exception_code,
                interrupt_bit,
                vcpu_ref.host_hart_id,
            ),
        },

        // Asynchronous interrupt
        causes::INTERRUPT => {
            match exception_code {
                // Timer interrupt
                interrupts::VIRTUAL_SUPERVISOR_TIMER => {
                    vcpu_ref.run_next_guest();
                }

                _ => fatal_exception::fatal_exception(
                    exception_code,
                    interrupt_bit,
                    vcpu_ref.host_hart_id,
                ),
            }
        }

        _ => unreachable!(),
    }
    unreachable!()
}
