use crate::types::{RgbaColor, CACHE_SIZE};

#[inline]
pub fn pixel_hash(pixel: RgbaColor) -> u8 {
    // index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
    let RgbaColor([r, g, b, a]) = pixel;
    // ((r as u32 * 5 + g as u32 * 11 + b as u32 * 13 + a as u32 * 17) % (CACHE_SIZE as u32)) as u8
    ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % CACHE_SIZE as u32) as u8
}

#[inline]
#[cold]
pub fn cold() {}

#[inline]
pub fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}
