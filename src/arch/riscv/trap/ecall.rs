use crate::registers;
use core::arch::asm;

#[inline(always)]
pub fn handle_ecall(vcpu_ref: &mut crate::vcpu::Vcpu) {
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
