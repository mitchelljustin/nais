use std::fmt::Display;

use crate::mem::addrs;

pub fn parse_hex(s: &str) -> Option<i32> {
    match i32::from_str_radix(s, 16) {
        Ok(val) => Some(val),
        Err(_) => None
    }
}


pub fn inst_loc_to_addr(loc: usize) -> i32 {
    loc as i32 + addrs::CODE_ENTRY
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

pub fn dump_errors<T: Display>(errors: &[T]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
