pub fn parse_hex(s: &str) -> Option<i32> {
    i32::from_str_radix(s, 16).ok()
}
