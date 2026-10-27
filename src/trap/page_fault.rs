use crate::{read_csr, vcpu::Vcpu, vcpu::Vr};
use core::cmp::Ordering;

#[inline(always)]
pub fn handle_load_store_page_fault(vcpu_ref: &mut Vcpu, exception_code: u64) {
    // Load/Store Guest Page Fault
    let requested_guest_addr = read_csr!("stval");

    let host_hart_id = vcpu_ref.host_hart_id;
    let guest_id_for_hart = vcpu_ref.guest_id_for_hart;

    let current_guest: &crate::Guest =
        crate::guest::get_current_guest(&host_hart_id, &guest_id_for_hart);

    // If the address is in the system MMIO window, process it; otherwise, allocate a page.
    let is_mmio = crate::SYSTEM_MMIO_WINDOWS
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
                core::arch::asm!("hlv.w {inst}, 0({addr})", inst = out(reg) instruction, addr = in(reg) executed_instruction_address);
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
        _ => panic!("Unsupported instruction opcode: {opcode:#x}"),
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
        _ => panic!("Unsupported instruction opcode: {opcode:#x}"),
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
