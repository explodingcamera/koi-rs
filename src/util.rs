#[macro_export]
macro_rules! color_hash {
    ($r:expr, $b:expr, $c:expr, $d:expr) => {
        // index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
        ($r as u32 * 3 + $b as u32 * 5 + $c as u32 * 7 + $d as u32 * 11) % 64
    };
}
