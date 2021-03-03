use std::iter;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

pub mod segs {
    use std::cmp;
    use std::ops::Range;

    pub struct Segment {
        pub name: &'static str,
        pub addr_range: Range<i32>,
    }

    impl Segment {
        pub const fn start(&self) -> i32 {
            self.addr_range.start
        }
        pub const fn end(&self) -> i32 {
            self.addr_range.end
        }

        pub fn contains(&self, addr: i32) -> bool {
            self.addr_range.contains(&addr)
        }

        pub fn clamp_range(&self, range: &mut Range<i32>) {
            range.start = cmp::max(range.start, self.addr_range.start);
            range.end = cmp::min(range.end, self.addr_range.end);
        }
    }

    pub const STACK: Segment = Segment {
        name: "stack",
        addr_range: 0x0_0000..0x1_0000, // 64 KiW
    };
    pub const CODE: Segment = Segment {
        name: "code",
        addr_range: 0x1_0000..0x2_0000, // 64 KiW
    };
    pub const DATA: Segment = Segment {
        name: "data",
        addr_range: 0x2_0000..0x4_0000, // 128 KiW
    };
    pub const ALL: &[&'static Segment] = &[
        &STACK,
        &CODE,
        &DATA,
    ];
    pub const ADDR_SPACE: Range<i32> = ALL[0].start()..ALL[ALL.len() - 1].end();
}

pub(crate) mod addrs {
    // Code constants
    pub const CODE_ENTRY: i32 = super::segs::CODE.start();

    // Stack addresses
    pub const PC: i32           = 0x00_00_00_00;
    pub const SP: i32           = 0x00_00_00_01;
    pub const FP: i32           = 0x00_00_00_02;
    pub const BOUNDARY: i32     = 0x00_00_00_03;

    // Stack initial values
    pub const INIT_PC: i32          = CODE_ENTRY;
    pub const INIT_SP: i32          = BOUNDARY + 1;
    pub const INIT_FP: i32          = 0x00_ff_ff_ff;
    pub const INIT_BOUNDARY: i32    = 0x00_bb_bb_bb;
}

pub fn inst_loc_to_addr(loc: usize) -> i32 {
    loc as i32 + addrs::CODE_ENTRY
}

pub struct Memory {
    vec: Vec<i32>,
}

impl Memory {
    pub fn new() -> Memory {
        let mut mem = Memory {
            vec: Vec::from_iter(iter::repeat(0).take(segs::ADDR_SPACE.len())),
        };
        // Initialize stack
        mem[addrs::PC]          = addrs::INIT_PC;
        mem[addrs::SP]          = addrs::INIT_SP;
        mem[addrs::FP]          = addrs::INIT_FP;
        mem[addrs::BOUNDARY]    = addrs::INIT_BOUNDARY;
        mem
    }
}

impl Index<i32> for Memory {
    type Output = i32;

    fn index(&self, index: i32) -> &Self::Output {
        &self.vec[index as usize]
    }
}

impl IndexMut<i32> for Memory {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.vec[index as usize]
    }
}