use crate::{println, vcpu::Vcpu};
use core::{
    arch::{asm, naked_asm},
    mem::offset_of,
};
const SXLEN: u8 = 64;

unsafe extern "C" {
    pub static __trampoline: [u8; 8];
    pub static __registers_load: [u64; 3];
    pub static __register_values: [u64; 3];
}

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
        crate::save_vector_registers(vcpu);

        vcpu_ref.vsstatus &= !(1 << 9 | 1 << 10);
        vcpu_ref.vsstatus |= 2 << 9;
    }

    // Save floating-point registers if floating-point extension turned on.
    let vsstatus_fs = (vsstatus & (1 << 13 | 1 << 14)) >> 13;

    if vsstatus_fs == 3 {
        crate::save_floating_point_registers(vcpu);

        vcpu_ref.vsstatus &= !(1 << 13 | 1 << 14);
        vcpu_ref.vsstatus |= 2 << 13;
    }

    let scause = vcpu_ref.scause;

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
        // Timer interrupt
        vcpu_ref.run_next_guest();
    } else if exception_code == 10 && interrupt_bit == 0 {
        // Ecall
        unsafe {
            match vcpu_ref.a7 {
                // Base extension
                0x10 => {
                    let a7 = vcpu_ref.a7;
                    let a6 = vcpu_ref.a6;
                    let mut a1 = vcpu_ref.a1;
                    let mut a0 = vcpu_ref.a0;
                    asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") a7, in("a6") a6);
                    vcpu_ref.a1 = a1;
                    vcpu_ref.a0 = a0;
                }
                // Legacy putchar extension
                0x01 => {
                    let a0 = vcpu_ref.a0;

                    asm!("ecall", in("a7") 0x01, in("a0") a0);
                }
                ecall_eid => {
                    panic!("Ecall EID ({ecall_eid}) cannot be processed.")
                }
            }
            vcpu_ref.sepc += 4;
            vcpu_ref.run();
        }
    } else if exception_code == 23 && interrupt_bit == 0 {
        // Store/AMO Guest Page Fault
        let requested_guest_addr = read_csr!("stval");

        let host_hart_id = vcpu_ref.host_hart_id;
        let guest_id_for_hart = vcpu_ref.guest_id_for_hart;

        let current_guest: &crate::Guest =
            crate::guest::get_current_guest(&host_hart_id, &guest_id_for_hart);

        // TODO: Optimize using a branchless algorithm without loops.
        // If this is an allowed MMIO address, perform a write to it; otherwise, allocate a page.
        for passthrough_mmio_addr in current_guest.passthrough_mmio_regions {
            if passthrough_mmio_addr[0] <= requested_guest_addr
                && requested_guest_addr <= (passthrough_mmio_addr[0] + passthrough_mmio_addr[1])
            {
                let executed_instruction_address: u64 = vcpu_ref.sepc;
                let mut instruction: u32;

                // Fetch an instruction by the address in stval.
                unsafe {
                    asm!("hlv.w {inst}, 0({addr})", inst = out(reg) instruction, addr = in(reg) executed_instruction_address);
                }

                // Get an instruction size in bytes by processing the quadrant.
                // If the lowest two bits are 0b11, the instruction is 4 bytes; otherwise, it is 2 bytes.
                let instruction_size = 2 + (((instruction & 0b10) >> 1) & (instruction & 0b01)) * 2;

                // Create an instruction sequence to perform the store requested by the guest.
                unsafe {
                    create_guest_store_sequence(instruction, instruction_size, vcpu);
                }

                // Execute the instruction sequence.
                unsafe {
                    asm!("fence.i", "jalr x1, 0({})", in(reg) core::ptr::addr_of!(__registers_load) as *mut [u32; 5], out("x1") _, );
                }

                // Return to the guest.
                vcpu_ref.sepc += instruction_size as u64;
                vcpu_ref.run();
            }
        }

        let locked_page_table = current_guest.page_table.lock();
        locked_page_table.map_and_allocate(
            requested_guest_addr,
            4096,
            crate::guest_table::PTE_R | crate::guest_table::PTE_W,
        );

        drop(locked_page_table);
        vcpu_ref.run();
    } else {
        panic!(
            "A trap occurred ({})\nexception code: (interrupt bit: {}) {}\nhart id: {}, vsstatus: {:#b}\nsstatus: {:#b}\nsepc: {:#x}\nstval: {:#x}",
            scause_str,
            interrupt_bit,
            exception_code,
            vcpu_ref.host_hart_id,
            read_csr!("vsstatus"),
            read_csr!("sstatus"),
            read_csr!("sepc"),
            read_csr!("stval"),
        );
    }
    unreachable!()
}

/// # Safety
/// This function is unsafe because it writes to RWX memory,
/// which could potentially allow arbitrary code execution.
#[inline(always)]
unsafe fn create_guest_store_sequence(instruction: u32, instruction_size: u32, vcpu: *const Vcpu) {
    let is_instruction_compressed = (4 - instruction_size) >> 1;

    // If the instruction is compressed, a NOP instruction is added
    // to its upper bytes to prevent it from being executed.
    let instruction = (instruction
        & (0x0000FFFFu32 | (0xFFFF0000u32 * (is_instruction_compressed ^ 1))))
        | ((0x0001u32 << 16u32) * is_instruction_compressed);

    // Extract the op (inst[1:0]) and funct3 (inst[15:13]) fields
    // from the compressed instruction code.
    let op = instruction & 0b11;
    let funct3 = (instruction >> 13) & 0x7;

    let is_inst_operands_size_5 =
                    // If op is 0b01 and funct3 is either 000, or 001, or 010, or 011.
                    (((((op ^ 0b11) & 2) >> 1) & (op & 1))
                    & ((funct3 >> 2) ^ 1))

                    // If op is 0b10, the register operand fields are 5 bits wide.
                    | (((op ^ 0b11) & 1) & (op >> 1))

                    // If op is 0b11, the instruction is not compressed
                    // and the register operand fields are 5 bits wide.
                    | ((op >> 1) & (op & 1));
    // Otherwise, register operand is 3 bits wide.

    let inst_operands_size = is_inst_operands_size_5 * 2 + 3;
    let operand_mask = (1 << inst_operands_size) - 1;

    // For 4-byte instructions, register offsets are rd=7, rs1=15, rs2=20.
    // For 2-byte instructions, offsets are rd/rs1=7, rs2=2.
    let rd_offset = 7;
    let rs1_offset = 7 + (is_instruction_compressed ^ 1) * 8;
    let rs2_offset = 2 + (is_instruction_compressed ^ 1) * 18;

    // Extract guest rd, rs1, rs2 operands from the instruction.
    let compressed_operand_offset =
        ((!inst_operands_size & 4) << 1) * ((instruction_size >> 2) ^ 1);
    let rd = ((instruction >> rd_offset) & operand_mask) + compressed_operand_offset;
    let rs1 = ((instruction >> rs1_offset) & operand_mask) + compressed_operand_offset;
    let rs2 = ((instruction >> rs2_offset) & operand_mask) + compressed_operand_offset;

    let return_address = 1;

    let gprs = vcpu as *const [u64; 32];

    // Write guest operands in memory to access them.
    unsafe {
        core::ptr::write_volatile(
            &raw const __register_values as *mut [u64; 3],
            [
                (*gprs)[rd as usize],
                (*gprs)[rs1 as usize],
                (*gprs)[rs2 as usize],
            ],
        );
    }

    // Calculate a free register to store the values of guest operands in it.
    let guest_regs =
        (!(3u32 | (1u32 << rd) | (1u32 << rs1) | (1u32 << rs2) | (1u32 << 2))).trailing_zeros();

    // Calculate the PC-relative address for guest operands.
    let pc_relative_values_addr = ((&raw const __register_values) as u64)
        - ((&raw const __registers_load) as u64

                    // Add the size of the instructions that precede the auipc instruction.
                         + 40u64);

    // Safely perform the guest store instruction.
    let buffer: [u32; 27] = [
        // Save the hypervisor registers in use

        // sd instruction_rd, 0(sp)
        (0x00013023 & !(0x1F << 20) | (rd << 20)),
        // addi sp, sp, -8 (stack allocation)
        0xff810113,
        // sd instruction_rs1, 0(sp)
        (0x00013023 & !(0x1F << 20) | (rs1 << 20)),
        // addi sp, sp, -8 (stack allocation)
        0xff810113,
        // sd instruction_rs2, 0(sp)
        (0x00013023 & !(0x1F << 20) | (rs2 << 20)),
        // addi sp, sp, -8 (stack allocation)
        0xff810113,
        // sd ra, 0(sp)
        (0x00013023 & !(0x1F << 20) | (return_address << 20)),
        // addi sp, sp, -8 (stack allocation)
        0xff810113,
        // sd guest_regs_ptr, 0(sp)
        (0x00013023 & !(0x1F << 20) | (guest_regs << 20)),
        // csrw sp, sscratch
        0x14011073,
        // Calculate the PC-relative address for guest operands.

        // auipc guest_regs,
        (0x00000017 & !(0x1F << 7)) | (guest_regs << 7),
        // addi guest_regs, guest_regs, ( pc relative guest_regs addr )
        (0x01a00013 & !(0xFFF << 20 | 0x1F << 15 | 0x1F << 7)
            | ((pc_relative_values_addr as u32) << 20 | guest_regs << 7 | guest_regs << 15)),
        // Load the guest context into the operand registers.

        // ld instruction_rd, 0(guest_regs)
        (0x00003003 & !((0x1F << 15) | (0x1F << 7)) | (guest_regs << 15 | rd << 7)),
        // ld instruction_rs1, 8(guest_regs)
        (0x00803003 & !((0x1F << 15) | (0x1F << 7)) | (guest_regs << 15 | rs1 << 7)),
        // ld instruction_rs2, 16(guest_regs)
        (0x01003003 & !((0x1F << 15) | (0x1F << 7)) | (guest_regs << 15 | rs2 << 7)),
        // Perform requested store instruction.

        // requested instruction
        instruction,
        // Restore the hypervisor context.

        // csrr sp, sscratch
        0x14002173,
        // ld guest_regs, 0(sp)
        (0x00013003 & !(0x1F << 7) | (guest_regs << 7)),
        // addi sp, sp, 8 (deallocate stack)
        0x00810113,
        // ld ra, 0(sp)
        (0x00013003 & !(0x1F << 7) | (return_address << 7)),
        // addi sp, sp, 8 (deallocate stack)
        0x00810113,
        // ld instruction_rs2, 0(sp)
        (0x00013003 & !(0x1F << 7) | (rs2 << 7)),
        // addi sp, sp, 8 (deallocate stack)
        0x00810113,
        // ld instrcuction_rs1, 0(sp)
        (0x00013003 & !(0x1F << 7) | (rs1 << 7)),
        // addi sp, sp, 8 (deallocate stack)
        0x00810113,
        // ld instructiond_rd, 0(sp)
        (0x00013003 & !(0x1F << 7) | (rd << 7)),
        // Return to the hypervisor.

        // jalr x0, 0(x1)
        0x000f0067 & !(0x1F << 15) | (return_address << 15),
    ];

    // Write the instruction sequence into the executable memory buffer.
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of!(__registers_load) as *mut [u32; 27],
            buffer,
        );
    }
}
