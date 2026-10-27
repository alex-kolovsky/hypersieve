use crate::read_csr;

#[inline(always)]
pub fn fatal_exception(exception_code: u64, interrupt_bit: u64, hart_id: usize) -> ! {
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

            _ => "Unsupported exception code",
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

            _ => "Unsupported exception code",
        }
    }
}
