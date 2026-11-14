#![no_std]
use core::sync::atomic::AtomicUsize;

use hypersieve_codegen::codegen_out_dir;

include!(concat!(codegen_out_dir!(), "/guest_abi_constants.rs"));

pub const GUEST_INDEXES: [usize; GUESTS_COUNT] = {
    let mut guest_indexes = [0; GUESTS_COUNT];
    let mut i = 0;
    while i != GUESTS_COUNT {
        guest_indexes[i] = i;
        i += 1;
    }
    guest_indexes
};

pub trait PageFaultInfoWrapper {}

pub struct GuestInfo {
    pub id: usize,
    pub active_hart_count: AtomicUsize,
    pub active_assigned_hart_count: AtomicUsize,
    pub hart_capacity: usize,
    pub assigned_hart_capacity: usize,
    pub assigned_harts: [Option<u32>; MAX_SUPPORTED_ASSIGNED_HARTS_PER_GUEST],
}
