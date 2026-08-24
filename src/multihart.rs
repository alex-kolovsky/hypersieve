use crate::{
    allocator::{BUDDY_ALLOCATOR, alloc_pages},
    println, read_csr,
};
use core::alloc::GlobalAlloc;
use core::arch::asm;

#[inline(always)]
fn sbi_hart_start(hart_id: usize, start_addr: usize, opaque: usize) -> (isize, isize) {
    let error: isize;
    let value: isize;

    unsafe {
        core::arch::asm!(
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

fn hart_init(_: usize, sp_end: usize) {
    let hart_id: usize;
    unsafe {
        core::arch::asm!(
            // Load stack pointer.
            "mv sp, {sp}",
            // Load trap handler function address.
            "la t0, trap_handler",
            "csrw stvec, t0",
            "mv {id}, a0",
            sp = in(reg) sp_end,
            id = out(reg) hart_id,
        );
    }

    println!("Hello from hart {hart_id}");
    hart_configure();

    panic!();
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
    let mut error_code: isize = 0;
    let mut value: isize = 0;
    let mut hart_id: usize = 0;
    let mut sp: *mut u8 = core::ptr::null_mut();
    let mut stack_size: usize = 0;

    while error_code == 0 {
        // Skip main hart.
        if hart_id != main_hart_id {
            // Allocate stack for hart.
            stack_size = 1024 * 1024;
            sp = alloc_pages(stack_size);
            let sp_end = (sp as usize) + stack_size;
            (error_code, value) = sbi_hart_start(hart_id, hart_init as *const () as usize, sp_end);
        }
        hart_id += 1;
    }

    // Error code will be equal to 3 if there's no core with that number, it means the other cores have started.
    if error_code != -3 {
        panic!("Failed to start harts.\nError code: {error_code}\nValue: {value}");
    } else {
        // Deallocate nonexistent hart stack.
        unsafe {
            BUDDY_ALLOCATOR.dealloc(
                sp,
                core::alloc::Layout::from_size_align(stack_size, 4096).unwrap(),
            );
        }
    }
}
