pub fn parse_hex(s: &str) -> Option<i32> {
    match i32::from_str_radix(s, 16) {
        Ok(val) => Some(val),
        Err(_) => None
    }
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