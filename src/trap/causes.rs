#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum Exception {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreAddressMisaligned = 6,
    StoreAccessFault = 7,
    EnvCallFromUMode = 8,
    EnvCallFromHSMode = 9,
    EnvCallFromVSMode = 10,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    InstructionGuestPageFault = 20,
    LoadGuestPageFault = 21,
    StoreGuestPageFault = 23,
}
