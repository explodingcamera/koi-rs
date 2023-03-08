pub fn color_hash(r: u8, b: u8, c: u8, d: u8) -> u32 {
    // index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
    (r as u32 * 3 + b as u32 * 5 + c as u32 * 7 + d as u32 * 11) % 64
}
