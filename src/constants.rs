pub const SEG_STACK_SIZE:   i32 = 0x2_0000; // 128 KiB
pub const SEG_CODE_SIZE:    i32 = 0x2_0000; // 128 KiB
#[allow(unused)]
pub const SEG_DATA_SIZE:    i32 = 0x4_0000; // 256 KiB

pub const SEG_STACK_START:  i32 = 0x0_0000;
pub const SEG_STACK_END:    i32 = SEG_STACK_START + SEG_STACK_SIZE;

pub const SEG_CODE_START:   i32 = SEG_STACK_END;
pub const SEG_CODE_END:     i32 = SEG_CODE_START + SEG_CODE_SIZE;

#[allow(unused)]
pub const SEG_DATA_START:   i32 = SEG_CODE_END;
#[allow(unused)]
pub const SEG_DATA_END:     i32 = SEG_DATA_START + SEG_DATA_SIZE;

// Stack constants
pub const PC_ADDR:          i32 = 0;
pub const SP_ADDR:          i32 = 1;
pub const FP_ADDR:          i32 = 2;
pub const BOUNDARY_ADDR:    i32 = 3;
pub const SP_MIN:           i32 = 4;
pub const INIT_STACK:       [i32; 4] = [
    SEG_CODE_START, // pc
    SP_MIN,         // sp
    0xffffff,       // fp
    0xbbbbbb        // boundary
];

pub const DEFAULT_MAX_CYCLES: usize = 10_000;
