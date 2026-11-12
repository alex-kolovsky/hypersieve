use core::arch::naked_asm;
use core::mem::offset_of;

use hypersieve_codegen::codegen_out_dir;

// Include architecture-sensitive files created by the build script.
include!(concat!(codegen_out_dir!(), "/vector_extension_support.rs"));
include!(concat!(
    codegen_out_dir!(),
    "/floating_point_extension_support.rs"
));

pub mod extension_routines;
mod guest_page_table;
mod hart_functions;
pub mod mmio;
mod registers;
mod trap;
mod vcpu;

// Export PageFaultInfo struct for the MMIO drivers.
pub use guest_page_table::GuestPageTable;
pub use guest_page_table::GuestPageTableFunctions;
pub use hart_functions::HartFunctions;
pub use trap::page_fault::PageFaultInfo;
pub use vcpu::Vcpu;

// Export trap handling functions.
pub use trap::process_exception;

use super::HardwareInterface;
pub struct Cpu;
impl HardwareInterface for Cpu {
    fn wait_for_interrupt() {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
