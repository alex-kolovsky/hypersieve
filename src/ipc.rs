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
    pub region: MemoryRegion,
}
