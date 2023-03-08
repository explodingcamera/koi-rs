use std::io::{Error, ErrorKind};

use num_enum::{IntoPrimitive, TryFromPrimitive};

// magic number to identify koi files
pub const MAGIC: &[u8] = b"KOI\xF0\x9F\x99\x82|\xF0\x9F\x99\x83";
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
