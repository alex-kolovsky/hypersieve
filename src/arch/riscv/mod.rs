use super::ArchCpu;

pub struct Cpu;
impl ArchCpu for Cpu {
    fn wait_for_interrupt() {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
