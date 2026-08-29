use crate::{
    GUESTS, HARTS, MAX_SUPPORTED_DEDICATED_GUESTS_PER_HART, allocator::alloc_pages, read_csr,
};
use core::{arch::asm, cell::UnsafeCell};

#[derive(Debug, Default)]
pub struct Hart {
    pub dedicated_to: UnsafeCell<[*mut crate::vcpu::Vcpu; MAX_SUPPORTED_DEDICATED_GUESTS_PER_HART]>,
    pub guests: [Option<usize>; MAX_SUPPORTED_DEDICATED_GUESTS_PER_HART],
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
        // Load the stack pointer.
        "mv sp, a1",
        // Load the trap handler function address.
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
        // Skip the main hart.
        if hart_id != main_hart_id {
            // Allocate a stack for a hart.
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
    if HARTS.len() <= hart_id {
        panic!("Excess hart, hart id: {hart_id}");
    }
    if is_hart_dedicated(hart_id) {
        // Run a guest on a dedicated hart.

        // Extract the guests for which this hart is dedicated.
        let hart = &crate::HARTS[hart_id];
        let guests = hart.guests;

        for (guest_id_in_dedicated_to, guest_id) in guests.iter().enumerate() {
            if let Some(guest_id) = guest_id {
                // Check if a guest is free. A guest is free if the dedicated hart count
                // is less than the active dedicated hart count.
                let is_free = crate::guest::assign_dedicated_vcpu_if_available(*guest_id);

                // Run a guest if it is free.
                if is_free.is_ok() {
                    unsafe {
                        let vcpu_ptr = (*hart.dedicated_to.get())[guest_id_in_dedicated_to];
                        (*vcpu_ptr).very_fisrt_run(hart_id, guest_id_in_dedicated_to);
                    }
                }
            } else {
                break;
            }
        }
    } else {
        // Run a guest on a non-dedicated hart.
        for global_guest_id in 0..GUESTS.len() {
            unsafe {
                let vcpu_ptr = crate::guest::assign_vcpu_if_available(global_guest_id);
                // Run a guest if it is free; otherwise, continue searching for a free guest.
                if let Ok(vcpu_ptr) = vcpu_ptr {
                    // Fill the guest ID field with the guest's position in the GUESTS static array.
                    (*vcpu_ptr).very_fisrt_run(hart_id, global_guest_id);
                }
            }
        }
    }

    // A hart is excess if there are no free guests for it.
    panic!("Couldn't find a job for a hart, hart id: {hart_id}");
}

pub fn is_hart_dedicated(hart_id: usize) -> bool {
    // Determine if a hart is dedicated. The dedicated_to
    // field must contain at least one non-null element.

    let dedicated_to = HARTS[hart_id].dedicated_to.get();

    unsafe {
        (*dedicated_to)
            .first()
            .filter(|ptr| !ptr.is_null())
            .is_some()
    }
}
