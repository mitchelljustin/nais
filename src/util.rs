use std::ops::Range;
use std::cmp;

pub fn parse_hex(s: &str) -> Option<i32> {
    match i32::from_str_radix(s, 16) {
        Ok(val) => Some(val),
        Err(_) => None
    }
}

pub fn clamp_range<T: Ord + Copy>(range: &mut Range<T>, clamp: Range<T>) {
    *range =
        cmp::max(range.start, clamp.start)..cmp::min(range.end, clamp.end);
}

#[macro_export]
macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            None => return None,
            Some(x) => x,
        }
    };
}