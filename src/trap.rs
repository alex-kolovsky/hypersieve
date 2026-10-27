use crate::{
    SYSTEM_MMIO_WINDOWS,
    registers::{self},
    vcpu::{Vcpu, Vr},
};
use core::{
    arch::{asm, naked_asm},
    cmp::Ordering,
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
    let vsstatus = vcpu_ref.vsstatus;

    // Save vector registers if vector extension turned on.
    let vsstatus_vs = (vsstatus & (1 << 9 | 1 << 10)) >> 9;

    if vsstatus_vs == 3 {
        crate::save_vrs(vcpu);

        vcpu_ref.vsstatus &= !(1 << 9 | 1 << 10);
        vcpu_ref.vsstatus |= 2 << 9;
    }

    // Save floating-point registers if floating-point extension turned on.
    let vsstatus_fs = (vsstatus & (1 << 13 | 1 << 14)) >> 13;

    if vsstatus_fs == 3 {
        crate::save_fprs(vcpu);

        vcpu_ref.vsstatus &= !(1 << 13 | 1 << 14);
        vcpu_ref.vsstatus |= 2 << 13;
    }

    let scause = vcpu_ref.scause;

    // Extract type of trap (exception/interrupt).
    let interrupt_bit = scause >> (SXLEN - 1);
    let exception_code: u64 = scause & !(1 << 63);

    match interrupt_bit {
        // Synchronous exception
        0 => match exception_code {
            // Ecall
            10 => handle_ecall(vcpu_ref),

            // Load guest-page fault or Store/AMO guest-page fault
            23 | 21 => handle_load_store_page_fault(vcpu_ref, exception_code),

            _ => unhandable_exception(exception_code, interrupt_bit, vcpu_ref.host_hart_id),
        },

        // Asynchronous interrupt
        1 => {
            match exception_code {
                // Timer interrupt
                6 => {
                    vcpu_ref.run_next_guest();
                }

                _ => unhandable_exception(exception_code, interrupt_bit, vcpu_ref.host_hart_id),
            }
        }

        _ => unreachable!(),
    }
    unreachable!()
}

#[inline(always)]
fn handle_load_store_page_fault(vcpu_ref: &mut Vcpu, exception_code: u64) {
    // Load/Store Guest Page Fault
    let requested_guest_addr = read_csr!("stval");

    let host_hart_id = vcpu_ref.host_hart_id;
    let guest_id_for_hart = vcpu_ref.guest_id_for_hart;

    let current_guest: &crate::Guest =
        crate::guest::get_current_guest(&host_hart_id, &guest_id_for_hart);

    // If the address is in the system MMIO window, process it; otherwise, allocate a page.
    let is_mmio = SYSTEM_MMIO_WINDOWS
        .binary_search_by(|region| {
            if requested_guest_addr < region[0] {
                Ordering::Greater
            } else if requested_guest_addr > region[0] + region[1] {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .is_ok();

    if is_mmio {
        process_mmio(vcpu_ref, requested_guest_addr, current_guest);
    } else {
        // If the guest tried to read a non-existent page, raise the page fault.
        if exception_code == 21 {
            inject_load_page_fault(vcpu_ref, requested_guest_addr)
        }

        // Otherwise, allocate a 4KiB-sized page for the guest.
        let locked_page_table = current_guest.page_table.lock();
        locked_page_table.map_and_allocate(
            requested_guest_addr,
            4096,
            crate::guest_table::PTE_R | crate::guest_table::PTE_W,
        );
        drop(locked_page_table);

        // Do not increment the guest SEPC CSR because no store operation was executed.
        // Return to the guest.
        vcpu_ref.run();
    }
}

#[inline(always)]
fn handle_ecall(vcpu_ref: &mut Vcpu) {
    // Ecall
    unsafe {
        match vcpu_ref.gprs[registers::A7_OFFSET] {
            // Base extension
            0x10 => {
                let a7 = vcpu_ref.gprs[registers::A7_OFFSET];
                let a6 = vcpu_ref.gprs[registers::A6_OFFSET];
                let mut a1 = vcpu_ref.gprs[registers::A1_OFFSET];
                let mut a0 = vcpu_ref.gprs[registers::A0_OFFSET];
                asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") a7, in("a6") a6);
                vcpu_ref.gprs[registers::A1_OFFSET] = a1;
                vcpu_ref.gprs[registers::A0_OFFSET] = a0;
            }
            // Legacy putchar extension
            0x01 => {
                let a0 = vcpu_ref.gprs[registers::A0_OFFSET];

                asm!("ecall", in("a7") 0x01, in("a0") a0);
            }
            ecall_eid => {
                panic!("Ecall EID ({ecall_eid}) cannot be processed.")
            }
        }

        // Advance the guest's PC by 4 bytes to step past the ecall instruction and prevent an infinite loop.
        vcpu_ref.sepc += 4;
        vcpu_ref.run();
    }
}

#[inline(always)]
fn unhandable_exception(exception_code: u64, interrupt_bit: u64, hart_id: usize) -> ! {
    let scause_str = scause_to_str(&exception_code, &interrupt_bit);
    panic!(
        "A trap occurred ({})\nexception code: (interrupt bit: {}) {}\nhart id: {hart_id}, vsstatus: {:#b}\nsstatus: {:#b}\nsepc: {:#x}\nstval: {:#x}",
        scause_str,
        interrupt_bit,
        exception_code,
        read_csr!("vsstatus"),
        read_csr!("sstatus"),
        read_csr!("sepc"),
        read_csr!("stval"),
    );
}

#[inline(always)]
fn scause_to_str<'a>(exception_code: &'a u64, interrupt_bit: &'a u64) -> &'a str {
    if *interrupt_bit == 1 {
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
    }
}

#[inline(always)]
fn process_mmio(vcpu_ref: &mut Vcpu, requested_guest_addr: u64, current_guest: &crate::Guest) {
    // If this is an allowed MMIO address, perform a write to it; otherwise, throw an exception to the guest.
    let search_result = current_guest
        .passthrough_mmio_regions
        .binary_search_by(|region| {
            if requested_guest_addr < region[0] {
                Ordering::Greater
            } else if requested_guest_addr > region[0] + region[1] {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });

    if let Ok(passthrough_mmio_id) = search_result {
        // Htinst CSR contains the transformed instruction that caused the fault.
        let htinst = read_csr!("htinst");

        // An htinst value of 0x0 indicates a non-transformable instruction (e.g., Vector Store);
        // Otherwise, instruction is scalar.
        if htinst == 0x0 {
            // Fetch the instruction from the address we got from the 'sepc' CSR.
            let executed_instruction_address = vcpu_ref.sepc;
            let mut instruction: u32;
            unsafe {
                asm!("hlv.w {inst}, 0({addr})", inst = out(reg) instruction, addr = in(reg) executed_instruction_address);
            }

            handle_vector_instruction(
                vcpu_ref,
                instruction,
                current_guest,
                passthrough_mmio_id,
                requested_guest_addr,
            );
        } else {
            handle_scalar_instruction(
                vcpu_ref,
                htinst,
                current_guest,
                requested_guest_addr,
                passthrough_mmio_id,
            );
        }

        // Get an instruction size in bytes by processing the quadrant.
        // If the lowest two bits are 0b11, the instruction is 4 bytes; otherwise, it is 2 bytes.
        let instruction_size = 2 + 2 * (((htinst & 0b11) == 0b11) as u32);

        // Increment the guest SEPC CSR to prevent an infinite loop.
        vcpu_ref.sepc += instruction_size as u64;
        vcpu_ref.run();
    } else {
        inject_store_page_fault(vcpu_ref, requested_guest_addr);
    }
}

#[inline(always)]
fn handle_scalar_instruction(
    vcpu_ref: &mut Vcpu,
    htinst: u64,
    current_guest: &crate::Guest,
    requested_guest_addr: u64,
    passthrough_mmio_id: usize,
) {
    // Get the rs2 field, the register identifier of the stored value.
    let rs2_id = ((htinst >> 20) & 0x1F) as usize;

    // Calculate the size from the value of the instruction's 'width' field.
    let width_field = (htinst >> 12) & 0x7;

    // Calculate the size in bytes of the value.
    // 0b00 => 1 Byte  (8-bit)
    // 0b01 => 2 Bytes (16-bit)
    // 0b10 => 4 Bytes (32-bit)
    // 0b11 => 8 Bytes (64-bit)
    let size = 1 << width_field;

    // If the size exceeds the allowed region, inject an exception into the guest.
    if size > current_guest.passthrough_mmio_regions[passthrough_mmio_id][1] {
        inject_store_page_fault(vcpu_ref, requested_guest_addr);
    }

    // Match the 7-bit opcode to determine the register file type (GPR vs. FPR)
    // and the memory access direction (Load vs. Store) of the trapped instruction.
    let opcode = htinst & 0x7F;
    match opcode {
        // Standard or compressed scalar integer store
        0x23 | 0x21 => {
            let value_to_store = vcpu_ref.gprs[rs2_id];
            scalar_write(size, requested_guest_addr, value_to_store);
        }

        // Standard or compressed scalar floating point store
        0x27 | 0x25 => {
            let value_to_store = vcpu_ref.fprs[rs2_id].to_bits();
            scalar_write(size, requested_guest_addr, value_to_store);
        }

        // Standard or compressed scalar integer load
        0x03 | 0x01 => {
            vcpu_ref.gprs[rs2_id] = scalar_load(size, requested_guest_addr);
        }

        // Standard or compressed scalar floating point load
        0x07 | 0x05 => {
            vcpu_ref.fprs[rs2_id] = f64::from_bits(scalar_load(size, requested_guest_addr));
        }
        _ => panic!("Unhandable opcode: {opcode:#x}"),
    };
}

#[inline(always)]
fn scalar_write(size: u64, requested_guest_addr: u64, value_to_store: u64) {
    // Write the value to the requested address.
    match size {
        1 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u8, value_to_store as u8);
        },
        2 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u16, value_to_store as u16);
        },
        4 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u32, value_to_store as u32);
        },
        8 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u64, value_to_store);
        },
        _ => {
            unreachable!();
        }
    }
}

#[inline(always)]
fn scalar_load(size: u64, requested_guest_addr: u64) -> u64 {
    // Write the value to the requested address.
    match size {
        1 => unsafe { core::ptr::read_volatile(requested_guest_addr as *mut u8) as u64 },
        2 => unsafe { core::ptr::read_volatile(requested_guest_addr as *mut u16) as u64 },
        4 => unsafe { core::ptr::read_volatile(requested_guest_addr as *mut u32) as u64 },
        8 => unsafe { core::ptr::read_volatile(requested_guest_addr as *mut u64) },
        _ => {
            unreachable!();
        }
    }
}

#[inline(always)]
fn handle_vector_instruction(
    vcpu_ref: &mut Vcpu,
    inst: u32,
    current_guest: &crate::Guest,
    passthrough_mmio_id: usize,
    requested_guest_addr: u64,
) {
    // Get the vs3 field, the register identifier of the stored value.
    let vs3_id = ((inst >> 7) & 0x1F) as usize;

    // Calculate the size based on the instruction's 'width' field encoding.
    let width_type = (inst >> 26) & 0x1;
    let size = match width_type {
        // Dynamic Mode; The size is in the 'vsew' field of 'vtype' CSR.
        1 => {
            // Calculate the size in bytes of the value stored.
            // 0b00 => 1 Byte  (8-bit)
            // 0b01 => 2 Bytes (16-bit)
            // 0b10 => 4 Bytes (32-bit)
            // 0b11 => 8 Bytes (64-bit)
            let vtype_vsew = (vcpu_ref.vtype >> 3) & 0x7;
            1 << vtype_vsew
        }
        // Static mode; The size is written in the 'width' field of the instruction.
        _ => {
            // Calculate the size in bytes of the value stored.
            // 0b00 => 1 Byte  (8-bit)
            // 0b01 => 2 Bytes (16-bit)
            // 0b10 => 4 Bytes (32-bit)
            // 0b11 => 8 Bytes (64-bit)
            let width_field = (inst >> 12) & 0x3;
            1 << width_field
        }
    };

    // If the size exceeds the allowed region, inject an exception into the guest.
    if size > current_guest.passthrough_mmio_regions[passthrough_mmio_id][1] {
        inject_store_page_fault(vcpu_ref, requested_guest_addr);
    }

    let opcode = inst & 0x7F;

    match opcode {
        // Vector store
        0x27 => {
            // Get the value to be stored from the array of saved vector registers.
            let value_to_store = vcpu_ref.vrs[vs3_id];
            vector_write(size, requested_guest_addr, value_to_store);
        }

        // Vector load
        0x07 => {
            vcpu_ref.vrs[vs3_id] = vector_load(size, requested_guest_addr);
        }
        _ => panic!("Unhandable opcode: {opcode:#x}"),
    }
}

#[inline(always)]
fn vector_write(size: u64, requested_guest_addr: u64, value_to_store: crate::vcpu::Vr) {
    // Write the value to the requested address.
    match size {
        1 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u8, value_to_store[0] as u8);
        },
        2 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u16, value_to_store[0] as u16);
        },
        4 => unsafe {
            core::ptr::write_volatile(requested_guest_addr as *mut u32, value_to_store[0]);
        },
        8 => unsafe {
            core::ptr::write_volatile(
                requested_guest_addr as *mut u64,
                // Combine two 32-bit elements of the vector register array to get a 64-bit value.
                ((value_to_store[0] as u64) << 32) | (value_to_store[0] as u64),
            );
        },
        _ => {
            unreachable!()
        }
    }
}

#[inline(always)]
fn vector_load(size: u64, requested_guest_addr: u64) -> Vr {
    let mut vr: Vr = [0; crate::VLEN / 32];
    match size {
        1 => unsafe {
            let bits = core::ptr::read_volatile(requested_guest_addr as *mut u8);
            vr[0] = bits as u32;
        },
        2 => unsafe {
            let bits = core::ptr::read_volatile(requested_guest_addr as *mut u16);
            vr[0] = bits as u32;
        },
        4 => unsafe {
            let bits = core::ptr::read_volatile(requested_guest_addr as *mut u32);
            vr[0] = bits;
        },
        8 => unsafe {
            let bits = core::ptr::read_volatile(requested_guest_addr as *mut u64);
            let lower = bits as u32;
            let upper = (bits >> 32) as u32;
            vr[0] = lower;
            vr[1] = upper;
        },
        _ => {
            unreachable!()
        }
    }
    vr
}

fn inject_store_page_fault(vcpu_ref: &mut Vcpu, requested_guest_addr: u64) -> ! {
    // Inject the Store/AMO Guest Page Fault

    // Set the failed instruction address to the guest's sepc (vsepc) CSR.
    vcpu_ref.vsepc = vcpu_ref.sepc;

    // Set the vscause CSR (guest's scause) to a 'Store/AMO page fault'.
    vcpu_ref.vscause = 15;

    // Set the failed address to guest's stval (vstval) CSR.
    vcpu_ref.vstval = requested_guest_addr;

    // Set the guest's PC to the guest's trap handler address (mask out the lower 2 bits which encode the 'Mode').
    vcpu_ref.sepc = vcpu_ref.vstvec & !0b11;

    // Return to the guest.
    vcpu_ref.run();
}

fn inject_load_page_fault(vcpu_ref: &mut Vcpu, requested_guest_addr: u64) -> ! {
    // Inject the Load Page Fault

    // Set the failed instruction address to the guest's sepc (vsepc) CSR.
    vcpu_ref.vsepc = vcpu_ref.sepc;

    // Set the vscause CSR (guest's scause) to a 'Load page fault'.
    vcpu_ref.vscause = 13;

    // Set the failed address to guest's stval (vstval) CSR.
    vcpu_ref.vstval = requested_guest_addr;

    // Set the guest's PC to the guest's trap handler address (mask out the lower 2 bits which encode the 'Mode').
    vcpu_ref.sepc = vcpu_ref.vstvec & !0b11;

    // Return to the guest.
    vcpu_ref.run();
}
