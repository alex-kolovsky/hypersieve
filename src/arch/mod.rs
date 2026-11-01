// RISC-V 64
#[cfg(target_arch = "riscv64")]
mod riscv;
#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(include_str!("riscv/asm/boot.S"));
#[cfg(target_arch = "riscv64")]
pub use riscv::*;

pub trait HardwareInterface {
    fn wait_for_interrupt();
}
