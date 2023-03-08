use num_enum::{IntoPrimitive, TryFromPrimitive};

// magic number to identify koi files
pub const MAGIC: &[u8] = b"KOI\xF0\x9F\x99\x82";
pub const MAX_PIXELS: usize = 4_000_000;
pub const MASK: u8 = 0xC0;
pub const CACHE_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub [u8; 4]);

impl RgbaColor {
    pub fn to_u32(&self) -> u32 {
        let RgbaColor([r, g, b, a]) = self;
        u32::from_be_bytes([*r, *g, *b, *a])
    }
    pub fn from_u32(color: u32) -> Self {
        let [r, g, b, a] = color.to_be_bytes();
        RgbaColor([r, g, b, a])
    }

    pub fn diff(&self, other: &Self) -> (u8, u8, u8) {
        let r = self.0[0].wrapping_sub(other.0[0]);
        let g = self.0[1].wrapping_sub(other.0[1]);
        let b = self.0[2].wrapping_sub(other.0[2]);
        (r, g, b)
    }
}

pub fn color_diff(diff: (u8, u8, u8)) -> Option<u8> {
    let r = diff.0.wrapping_add(2);
    let g = diff.1.wrapping_add(2);
    let b = diff.2.wrapping_add(2);

    match r | g | b {
        0x00..=0x03 => Some(Op::Diff as u8 | (r << 4) as u8 | (g << 2) as u8 | b as u8),
        _ => None,
    }
}

pub fn luma_diff(diff: (u8, u8, u8)) -> Option<[u8; 2]> {
    let r = diff.0.wrapping_add(8).wrapping_sub(diff.1);
    let g = diff.0.wrapping_add(32);
    let b = diff.0.wrapping_add(8).wrapping_sub(diff.1);

    match (r | b, g) {
        (0x00..=0x0F, 0x00..=0x3F) => Some([Op::Luma as u8 | g, r << 4 | b]),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u8)]
pub enum Op {
    Index = 0x00,
    Diff = 0x40,
    Luma = 0x80,
    Run = 0xC0,
    Rgb = 0xfe,
    Rgba = 0xff,
}

#[derive(IntoPrimitive, TryFromPrimitive, Debug)]
#[repr(u32)]
pub enum Compression {
    None = 0,
    Lz4 = 1,
}

#[derive(IntoPrimitive, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Channels {
    Gray = 1,
    Rgb = 3,
    Rgba = 4,
}

#[derive(IntoPrimitive, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Colorspace {
    Srgb,
}
