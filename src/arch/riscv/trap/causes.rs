#![allow(dead_code)]
pub const EXCEPTION: u64 = 0;
pub const INTERRUPT: u64 = 1;

pub mod exceptions {
    pub const INSTRUCTION_ADDRESS_MISALIGNED: u64 = 0;
    pub const INSTRUCTION_ACCESS_FAULT: u64 = 1;
    pub const ILLEGAL_INSTRUCTION: u64 = 2;
    pub const BREAKPOINT: u64 = 3;
    pub const LOAD_ADDRESS_MISALIGNED: u64 = 4;
    pub const LOAD_ACCESS_FAULT: u64 = 5;
    pub const STORE_ADDRESS_MISALIGNED: u64 = 6;
    pub const STORE_ACCESS_FAULT: u64 = 7;
    pub const ENV_CALL_FROM_U_MODE: u64 = 8;
    pub const ENV_CALL_FROM_HS_MODE: u64 = 9;
    pub const ENV_CALL_FROM_VS_MODE: u64 = 10;
    pub const INSTRUCTION_PAGE_FAULT: u64 = 12;
    pub const LOAD_PAGE_FAULT: u64 = 13;
    pub const STORE_PAGE_FAULT: u64 = 15;

    pub const INSTRUCTION_GUEST_PAGE_FAULT: u64 = 20;
    pub const LOAD_GUEST_PAGE_FAULT: u64 = 21;
    pub const STORE_GUEST_PAGE_FAULT: u64 = 23;
}

pub mod interrupts {
    pub const SUPERVISOR_SOFTWARE: u64 = 1;
    pub const VIRTUAL_SUPERVISOR_SOFTWARE: u64 = 2;
    pub const SUPERVISOR_TIMER: u64 = 5;
    pub const VIRTUAL_SUPERVISOR_TIMER: u64 = 6;
    pub const SUPERVISOR_EXTERNAL: u64 = 9;
    pub const VIRTUAL_SUPERVISOR_EXTERNAL: u64 = 10;
}
