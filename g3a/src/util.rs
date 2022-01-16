/// Writes the given String into the Target-Buffer
pub fn write_string(target: &mut [u8], content: &str) {
    let content_length = content.len();
    if content_length > target.len() {
        return;
    }

    target[0..content_length].copy_from_slice(content.as_bytes());
}

/// Calculates a single Checksum by adding together
/// the given Byte-Sequence
pub fn checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for elem in data.iter() {
        sum = sum.wrapping_add(*elem as u32);
    }
    sum
}
