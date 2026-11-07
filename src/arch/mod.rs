// RISC-V 64
#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(include_str!("riscv64/asm/boot.S"));
#[cfg(target_arch = "riscv64")]
pub use riscv64::*;
#[cfg(target_arch = "riscv64")]
#[path = "riscv64/uart.rs"]
pub mod uart;

pub trait HardwareInterface {
    fn wait_for_interrupt();
}

pub trait VcpuInterface {
    fn host_stack_pointer(&self) -> u64;
    fn host_hart_id(&self) -> usize;
    fn current_guest_id_for_hart(&self) -> usize;
    fn current_guest_ref(&self) -> &'static crate::Guest;
    fn prepare_guest_vcpu_for_hart_switch(
        &mut self,
        host_sp: u64,
        host_hart_id: usize,
        guest_id_for_hart: usize,
    );

    fn new(guest_entry: u64, virtual_hart_id: u64) -> Self
    where
        Self: Sized;

    fn run(&mut self) -> !;
    fn very_fisrt_run(&mut self, host_hart_id: usize, guest_id_for_hart: usize) -> !;

    fn initialize_vcpu(
        &mut self,
        page_table: crate::arch::GuestPageTable,
        guest: &crate::Guest,
        vcpu_ptr_index: usize,
    );
}

pub trait HartInterface {
    fn start_harts(main_hart_id: usize);
    fn hart_configure();
}

pub trait GuestPageTableInterface {
    fn map_kernel(guest_entry_gpa: usize, kernel_memory: *mut u8) -> GuestPageTable;
}
