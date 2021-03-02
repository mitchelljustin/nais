use std::ops::{Index, IndexMut};
use std::iter;
use std::iter::FromIterator;

pub mod segs {
    use std::ops::Range;
    use std::cmp;

    pub struct Segment {
        pub name: &'static str,
        pub addr_range: Range<i32>,
    }
    impl Segment {
        pub fn len(&self) -> usize {
            self.addr_range.len()
        }

        pub const fn start(&self) -> i32 {
            self.addr_range.start
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
        addr_range: 0x0_0000..0x1_0000, // 64 KiB
    };
    pub const CODE: Segment = Segment {
        name: "code",
        addr_range: 0x1_0000..0x3_0000, // 128 KiB
    };
    pub const DATA: Segment = Segment {
        name: "data",
        addr_range: 0x3_0000..0x8_0000, // 320 KiB
    };
    pub const ALL: &[&'static Segment] = &[
        &STACK,
        &CODE,
        &DATA,
    ];
}

pub(crate) mod addrs {
    // Stack constants
    pub const PC:          i32 = 0x00_00_00_00;
    pub const SP:          i32 = 0x00_00_00_01;
    pub const FP:          i32 = 0x00_00_00_02;
    pub const BOUNDARY:    i32 = 0x00_00_00_03;
    pub const INIT_SP:     i32 = 0x00_00_00_04;
    // Code constants
    pub const CODE_ENTRY:  i32 = super::segs::CODE.start();
}


pub struct Memory {
    mem: Vec<i32>,
}

impl Memory {
    pub fn new() -> Memory {
        let total_len = segs::ALL.iter().map(|s| s.len()).sum();
        let mut mem = Memory {
            mem: Vec::from_iter(iter::repeat(0).take(total_len)),
        };
        // Initialize stack
        mem[addrs::PC]         = addrs::CODE_ENTRY;
        mem[addrs::SP]         = addrs::INIT_SP;
        mem[addrs::FP]         = 0xff_ff_ff;
        mem[addrs::BOUNDARY]   = 0xbb_bb_bb;
        mem
    }
}

impl Index<i32> for Memory {
    type Output = i32;

    fn index(&self, index: i32) -> &Self::Output {
        &self.mem[index as usize]
    }
}

impl IndexMut<i32> for Memory {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.mem[index as usize]
    }
}