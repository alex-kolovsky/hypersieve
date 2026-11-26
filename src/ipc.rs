use crate::guest::MemoryRegion;

#[derive(Debug)]
pub struct SharedBuffer {
    pub writer_guest_id: usize,
    pub reader_guest_id: usize,
    pub region: MemoryRegion,
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SharedBufferPolicy {
    Write,
    Read(NotifyPolicy),
}

impl SharedBufferPolicy {
    fn policy_bit(&self) -> usize {
        match self {
            SharedBufferPolicy::Read(_) => 0,
            SharedBufferPolicy::Write => 1,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NotifyPolicy {
    Poll = 0,
    Synchronous = 1,
}

#[derive(Debug)]
pub struct SharedBufferInfo {
    pub global_id: usize,
    pub policy: SharedBufferPolicy,
    pub region: MemoryRegion,
}
