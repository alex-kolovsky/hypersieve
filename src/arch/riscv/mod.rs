mod trap;

// Export PageFaultInfo struct for the MMIO drivers.
pub use trap::page_fault::PageFaultInfo;

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
