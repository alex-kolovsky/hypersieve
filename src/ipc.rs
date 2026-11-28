use crate::guest::MemoryRegion;

#[derive(Debug)]
pub struct SharedBuffer {
    pub writer_guest_id: usize,
    pub reader_guest_id: usize,
    pub region: MemoryRegion,
    pub writer_guest_type: GuestType,
    pub reader_guest_type: GuestType,
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SharedBufferPolicy {
    Write(&'static GuestType),
    Read(&'static GuestType),
}

impl SharedBufferPolicy {
    fn policy_bit(&self) -> usize {
        match self {
            SharedBufferPolicy::Read(_) => 0,
            SharedBufferPolicy::Write(_) => 1,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GuestType {
    Native = 0,
    Wasm = 1,
}

#[derive(Debug)]
pub struct SharedBufferInfo {
    pub global_id: usize,
    pub policy: SharedBufferPolicy,
}
