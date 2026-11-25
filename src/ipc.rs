use crate::guest::MemoryRegion;

#[derive(Debug)]
pub struct SharedBuffer {
    pub writer_guest_id: usize,
    pub reader_guest_id: usize,
    pub region: MemoryRegion,
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SharedBufferPolicies {
    Write = 0,
    Read = 1,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SharedBufferNotifyType {
    Poll = 0,
    Synchronous = 1,
}

#[derive(Debug)]
pub struct SharedBufferInfo {
    pub global_id: usize,
    pub policy: SharedBufferPolicies,
    pub size: usize,
    pub notify_type: SharedBufferNotifyType,
}

impl SharedBufferInfo {
    // Bits 0..=1  - memory policy.
    // Bits 2..=4  - notify type.
    // Bits 5..=
    // Bits 63..=5 - reserved.
    pub fn pack_ipc_info(&self) -> u64 {
        let notify_type_bits = (self.notify_type as u64) << 2;

        (self.policy as u64) | notify_type_bits
    }
}
