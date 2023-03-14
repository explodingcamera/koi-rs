use crate::types::RgbaColor;

pub fn pixel_hash(pixel: RgbaColor) -> u8 {
    // index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
    let RgbaColor([r, g, b, a]) = pixel;
    ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % 64) as u8
}
