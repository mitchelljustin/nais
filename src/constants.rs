pub const SEG_LEN:          i32 = 0x1_0000; // 64KiB

pub const SEG_STACK_START:  i32 = 0x0_0000;
pub const SEG_STACK_END:    i32 = SEG_STACK_START + SEG_LEN;

pub const PC_ADDR: i32 = 0;
pub const SP_ADDR: i32 = 1;
pub const FP_ADDR: i32 = 2;
pub const BOUNDARY_ADDR: i32 = 3;
pub const INIT_STACK: &[i32] = &[SEG_CODE_START, 4, 0xffffff, 0xbbbbbb]; // PC, SP, FP, boundary
pub const SP_MINIMUM: i32 = 4;

pub const SEG_CODE_START:   i32 = SEG_STACK_END;
pub const SEG_CODE_END:     i32 = SEG_CODE_START + SEG_LEN;

// pub const SEG_DATA_START:   i32 = SEG_CODE_END;
// pub const SEG_DATA_END:     i32 = SEG_DATA_START + SEG_LEN;

pub const MAX_CYCLES: usize = 10_000;
