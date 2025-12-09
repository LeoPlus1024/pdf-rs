pub(crate) fn literal_to_u64(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut value: u64 = 0;
    for i in 0..len {
        let b = bytes[i] - 48;
        value = (value * 10) + b as u64;
    }
    value
}

#[inline]
pub(crate) fn line_ending(b: u8) -> bool {
    b == 10 || b == 13
}
