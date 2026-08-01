const SXLEN: u8 = 64;

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
extern "C" fn trap_handler() -> ! {
    let scause = read_csr!("scause");

    // Extract type of trap (exception/interrupt)
    let interrupt_bit = scause >> (SXLEN - 1);
    let exception_code: u64 = scause & !(1 << 63);

    let scause_str: &str = if interrupt_bit == 1 {
        match exception_code {
            1 => "Supervisor software interrupt",
            5 => "Supervisor timer interrupt",
            9 => "Supervisor external interrupt",
            13 => "Counter-overflow interrupt",

            _ => "Unhandable exeption code",
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
            8 => "Environment call from U-mode",
            9 => "Environment call from S-mode",
            12 => "Instruction page fault",
            13 => "Load page fault",
            15 => "Store/AMO page fault",
            18 => "Software check",
            19 => "Hardware error",

            _ => "Unhandable exeption code",
        }
    };

    panic!("A trap occurred ({})", scause_str);
}
