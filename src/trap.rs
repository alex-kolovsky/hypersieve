use crate::{println, vcpu::Vcpu};
use core::{
    arch::{asm, naked_asm},
    mem::offset_of,
};
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

        // Restore a0 from sscratch, and save in to VCpu.
        "csrr t0, sscratch",
        "sd t0, {a0_offset}(a0)",

        // Switch to the hypervisor's stack.
        "ld sp, {host_sp_offset}(a0)",

        // a0 (first argument) is still the vcpu pointer here.
        "call {handle_trap}",
        handle_trap = sym handle_trap,
        //CSRs

        scause_offset = const offset_of!(Vcpu, scause),
        sepc_offset = const offset_of!(Vcpu, sepc),
        hstatus_offset = const offset_of!(Vcpu, hstatus),
        vsstatus_offset = const offset_of!(Vcpu, vsstatus),
        vstvec_offset = const offset_of!(Vcpu, vstvec),
        vsscratch_offset = const offset_of!(Vcpu, vsscratch),
        vsepc_offset = const offset_of!(Vcpu, vsepc),
        vscause_offset = const offset_of!(Vcpu, vscause),
        vstval_offset = const offset_of!(Vcpu, vstval),
        vsie_offset = const offset_of!(Vcpu, vsie),
        vsatp_offset = const offset_of!(Vcpu, vsatp),
        vstimecmp_offset = const offset_of!(Vcpu, vstimecmp),
        host_sp_offset = const offset_of!(Vcpu, host_sp),

        // GPRs
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
pub fn handle_trap(vcpu: *mut Vcpu) {
    let vsstatus = unsafe { (*vcpu).vsstatus };

    // Save vector registers if vector extension turned on.
    let vsstatus_vs = (vsstatus & (1 << 9 | 1 << 10)) >> 9;

    if vsstatus_vs == 3 {
        crate::save_vector_registers(vcpu);

        unsafe {
            (*vcpu).vsstatus &= !(1 << 9 | 1 << 10);
            (*vcpu).vsstatus |= 2 << 9;
        }
    }

    let scause = unsafe { (*vcpu).scause };

    // Extract type of trap (exception/interrupt).
    let interrupt_bit = scause >> (SXLEN - 1);
    let exception_code: u64 = scause & !(1 << 63);

    let scause_str: &str = if interrupt_bit == 1 {
        match exception_code {
            1 => "Supervisor software interrupt",
            2 => "Virtual supervisor software interrupt",
            3 => "Machine software interrupt",
            5 => "Supervisor timer interrupt",
            6 => "Virtual supervisor timer interrupt",
            7 => "Machine timer interrupt",

            9 => "Supervisor external interrupt",
            10 => "Virtual supervisor external interrupt",
            11 => "Machine external interrupt",
            12 => "Supervisor guest external interrupt",
            13 => "Counter-overflow interrupt",

            _ => "Unhandable exception code",
        }
    } else {
        match exception_code {
            0 => "Instruction address misaligned",
            1 => "Instruction access fault",
            2 => "Illegal instruction",
            3 => "Breakpoint",
            4 => "Load address misaligned",
            5 => "Load access fault",
            6 => "Store/AMO address misaligned",
            7 => "Store/AMO access fault",
            8 => "Environment call from U-mode or VU-mode",
            9 => "Environment call from HS-mode",
            10 => "Environment call from VS-mode",
            11 => "Environment call from M-mode",
            12 => "Instruction page fault",
            13 => "Load page fault",
            15 => "Store/AMO page fault",
            16 => "Double trap",
            18 => "Software check",
            19 => "Hardware error",
            20 => "Instruction guest-page fault",
            21 => "Load guest-page fault",
            22 => "Virtual instruction",
            23 => "Store/AMO guest-page fault",

            _ => "Unhandable exception code",
        }
    };

    if exception_code == 6 && interrupt_bit == 1 {
        unsafe {
            (*vcpu).run_next_guest();
        }
    } else if exception_code == 10 && interrupt_bit == 0 {
        unsafe {
            match (*vcpu).a7 {
                // Base extension
                0x10 => {
                    let a7 = (*vcpu).a7;
                    let a6 = (*vcpu).a6;
                    let mut a1 = (*vcpu).a1;
                    let mut a0 = (*vcpu).a0;
                    asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") a7, in("a6") a6);
                    (*vcpu).a1 = a1;
                    (*vcpu).a0 = a0;
                }
                // Legacy putchar extension
                0x01 => {
                    let a0 = (*vcpu).a0;

                    asm!("ecall", in("a7") 0x01, in("a0") a0);
                }
                ecall_eid => {
                    panic!("Ecall EID ({ecall_eid}) cannot be processed.")
                }
            }
            (*vcpu).sepc += 4;
            (*vcpu).run();
        }
    } else {
        println!(
            "A trap occurred ({})\nexception code: (interrupt bit: {}) {}\nhart id: {}, vsstatus: {:#b}\nsstatus: {:#b}\nsepc: {:#x}",
            scause_str,
            interrupt_bit,
            exception_code,
            unsafe { (*vcpu).host_hart_id },
            read_csr!("vsstatus"),
            read_csr!("sstatus"),
            read_csr!("sepc"),
        );
    }
    panic!();
}
