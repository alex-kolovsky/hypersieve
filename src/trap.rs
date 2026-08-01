use core::arch::asm;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".stvec")]
extern "C" fn trap_handler() -> ! {
    let scause: u64;
    unsafe {
        asm!("csrr {}, scause", out(reg) scause);
    }

    panic!("scause: {scause}");
}
