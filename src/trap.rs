use core::arch::asm;

const SXLEN: u8 = 64;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".stvec")]
extern "C" fn trap_handler() -> ! {
    let scause: u64;
    unsafe {
        asm!("csrr {}, scause", out(reg) scause);
    }

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

    panic!("An exception occurred ({})", scause_str);
}
