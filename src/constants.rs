pub const CODE_START: i32 = 0x1_0000;
pub const CODE_MAX_LEN: usize = CODE_START as usize;

pub const PC_ADDR: i32 = 0;
pub const SP_ADDR: i32 = 1;
pub const FP_ADDR: i32 = 2;
pub const BOUNDARY_ADDR: i32 = 3;
pub const INIT_STACK: &[i32] = &[CODE_START, 4, 0xffffff, 0xbbbbbb]; // PC, SP, FP, boundary

pub const MAX_CYCLES: usize = 10_000;
