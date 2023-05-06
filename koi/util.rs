// #[inline]
// pub fn pixel_hash<const C: usize>(pixel: RgbaColor<C>) -> u8 {
//     // index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
//     let (r, g, b, a) = (pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);
//     // ((r as u32 * 5 + g as u32 * 11 + b as u32 * 13 + a as u32 * 17) % (CACHE_SIZE as u32)) as u8
//     ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % CACHE_SIZE as u32) as u8
// }

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
