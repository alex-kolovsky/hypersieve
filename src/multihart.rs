use crate::{MAX_SUPPORTED_DEDICATED_GUESTS_PER_HART, allocator::alloc_pages, read_csr};
use {core::arch::asm, core::sync::atomic::Ordering};

#[derive(Debug, Default)]
pub struct Hart {
    pub dedicated_to:
        core::cell::UnsafeCell<[*mut crate::vcpu::Vcpu; MAX_SUPPORTED_DEDICATED_GUESTS_PER_HART]>,
}

// The Hart struct is thread-safe if we never change dedicated_to field after waking the harts up.
unsafe impl Sync for Hart {}
unsafe impl Send for Hart {}

#[inline(always)]
fn sbi_hart_start(hart_id: usize, start_addr: usize, opaque: usize) -> (isize, isize) {
    let error: isize;
    let value: isize;

    unsafe {
        asm!(
            "ecall",
            in("a0") hart_id,
            in("a1") start_addr,
            in("a2") opaque,
            in("a6") 0,
            in("a7") 0x48534D,
            lateout("a0") error,
            lateout("a1") value,
        );
    }

    (error, value)
}

#[unsafe(naked)]
pub extern "C" fn hart_entry() {
    core::arch::naked_asm!(
        // Load stack pointer.
        "mv sp, a1",
        // Load trap handler function address.
        "la t0, trap_handler",
        "csrw stvec, t0",
        "j hart_init",
    );
}

#[unsafe(no_mangle)]
pub fn hart_init(hart_id: usize) -> ! {
    hart_configure();
    jump_in_guest(hart_id);
}

#[inline(always)]
pub fn hart_configure() {
    // Set the VS Timer Interrupt Enable (VSTIE) bit in hie to allow timer interrupts in VS mode.
    let hie_vstie = 1 << 6;
    let hie = read_csr!("hie") | hie_vstie; // hie: Hypervisor Interrupt Enable

    // Set the Supervisor Timer Interrupt Enable (STIE) bit in vsie to handle guest timer interrupts.
    let vsie_stie = 1 << 6;
    let vsie = read_csr!("vsie") | vsie_stie; // vsie: Virtual Supervisor Interrupt Enable

    // Set the STimecmp Enable (STCE) bit in henvcfg to enable S/VS mode time comparators.
    let henvcfg_stce = 1 << 63;
    let henvcfg = read_csr!("henvcfg") | henvcfg_stce; // henvcfg: Hypervisor Environment Configuration

    unsafe {
        asm!(
            "csrw hie, {hie}",
            "csrw henvcfg, {henvcfg}",
            "csrw vsie, {vsie}",
            hie = in(reg) hie,
            henvcfg = in(reg) henvcfg,
            vsie = in(reg) vsie,
        );
    }
}

#[inline(always)]
pub fn start_harts(main_hart_id: usize) {
    let harts_count = number_of_harts();
    let mut error_code: isize = 0;
    let mut value: isize = 0;
    let stack_size = 1024 * 1024;
    let mut sp: *mut u8;

    for hart_id in 0..harts_count {
        // Skip main hart.
        if hart_id != main_hart_id {
            // Allocate stack for hart.
            sp = alloc_pages(stack_size);
            let sp_end = (sp as usize) + stack_size;
            (error_code, value) = sbi_hart_start(hart_id, hart_entry as *const () as usize, sp_end);
        }
    }

    if error_code != 0 {
        panic!("Error occurred while waking up harts\nError code: {error_code}\nValue: {value}");
    }
}

pub fn number_of_harts() -> usize {
    let mut id: usize = 0;
    let mut error_code: usize = 0;
    loop {
        unsafe {
            asm!(
                "ecall",
                in("a0") id,
                in("a7") 0x48534D,
                in("a6") 0x02,
                lateout("a0") error_code,
            );
        }

        if error_code != 0 {
            break;
        }

        id += 1;
    }

    id
}

pub fn jump_in_guest(hart_id: usize) -> ! {
    unsafe {
        let first_dedicated_vcpu_ptr = (*crate::HARTS[hart_id].dedicated_to.get())[0];
        if !(first_dedicated_vcpu_ptr).is_null() {
            (*first_dedicated_vcpu_ptr).very_fisrt_run(hart_id);
        }
    }

    for guest in crate::GUESTS.iter() {
        let harts = guest.active_hart_count.fetch_add(1, Ordering::Relaxed);
        if harts < guest.harts_cap {
            unsafe {
                let vcpu_ptr: *mut crate::vcpu::Vcpu =
                    (&mut *guest.vcpu_ptrs.lock())[harts].unwrap();
                (*vcpu_ptr).very_fisrt_run(harts);
            }
        }
    }

    panic!("Excess hart, hart id: {hart_id}");
}
