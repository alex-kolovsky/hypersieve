#![no_main]
#![no_std]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    hypersieve::print_and_wfi(info);
}

#[unsafe(no_mangle)]
pub extern "C" fn main(hart_id: usize) -> ! {
    hypersieve::run(hart_id);
}
