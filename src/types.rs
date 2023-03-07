use std::io::{Error, ErrorKind};

// magic number to identify koi files
pub const MAGIC: &[u8] = b"KOI\xF0\x9F\x99\x82|\xF0\x9F\x99\x83";

pub const MAX_PIXELS: usize = 4_000_000;
pub const OP_INDEX: u8 = 0x00;
pub const OP_DIFF: u8 = 0x40;
pub const OP_LUMA: u8 = 0x80;
pub const OP_RUN: u8 = 0xC0;
pub const OP_RGB: u8 = 0xfe;
pub const OP_RGBA: u8 = 0xff;
pub const OP_MASK: u8 = 0xC0;
pub const PADDING: u8 = 0b00000001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub u8, pub u8, pub u8, pub u8);
impl RgbaColor {
    pub fn to_u32(&self) -> u32 {
        let RgbaColor(r, g, b, a) = self;
        u32::from_be_bytes([*r, *g, *b, *a])
    }
    pub fn from_u32(color: u32) -> Self {
        let [r, g, b, a] = color.to_be_bytes();
        RgbaColor(r, g, b, a)
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Gray = 1,
    Rgb = 3,
    Rgba = 4,
}

impl TryFrom<u32> for Channels {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Channels::Gray),
            3 => Ok(Channels::Rgb),
            4 => Ok(Channels::Rgba),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid number of channels: {}", value),
            )),
        }
    }
}
