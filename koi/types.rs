use num_enum::{IntoPrimitive, TryFromPrimitive};

// magic number to identify koi files
pub const MAGIC: [u8; 8] = *b"KOI \xF0\x9F\x99\x82";
pub const MASK: u8 = 0xC0;
pub const CACHE_SIZE: usize = 62; // -2 seems to give better compression on average than 64
pub const END_OF_IMAGE: [u8; 8] = *b"\x00\x00\x00\x00\xF0\x9F\x99\x82";

pub const OP_INDEX: u8 = 0x00;
pub const OP_INDEX_END: u8 = 0x3F;
pub const OP_DIFF: u8 = 0x40;
pub const OP_DIFF_END: u8 = 0x40 | 0x3F;
pub const OP_LUMA: u8 = 0x80;
pub const OP_LUMA_END: u8 = 0x80 | 0x3F;
pub const OP_DIFF_ALPHA: u8 = 0xC0;
pub const OP_DIFF_ALPHA_END: u8 = 0xC0 | 0x3b; // we only have 59 possible values for diff alpha so we can use the color opcodes

// pub const OP_RUNLENGTH: u8 = 0xC0;
// pub const OP_RUNLENGTH_END: u8 = 0xfb;

pub const OP_GRAY: u8 = 0xfc;
pub const OP_GRAY_ALPHA: u8 = 0xfd;

pub const OP_RGB: u8 = 0xfe;
pub const OP_RGBA: u8 = 0xff;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u8)]
pub enum Op {
    Index = OP_INDEX,
    Diff = OP_DIFF,
    Luma = OP_LUMA,
    // Run = OP_RUNLENGTH,
    DiffAlpha = OP_DIFF_ALPHA,
    Gray = OP_GRAY,
    GrayAlpha = OP_GRAY_ALPHA,
    Rgb = OP_RGB,
    Rgba = OP_RGBA,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub [u8; 4]);

impl RgbaColor {
    pub fn is_gray(&self) -> bool {
        let RgbaColor([r, g, b, _]) = self;
        r == g && g == b
    }

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

    pub fn alpha_diff(&self, other: &Self) -> Option<u8> {
        let a1 = self.0[3];
        let a2 = other.0[3];
        let diff = a2.wrapping_sub(a1).wrapping_add(0x1e);

        match diff {
            0x00..=0x3b => Some(Op::DiffAlpha as u8 | diff),
            _ => None,
        }
    }

    pub fn apply_alpha_diff(&self, op: u8) -> Self {
        let diff = (op & !(Op::DiffAlpha as u8)).wrapping_sub(0x1e);
        let new_alpha = self.0[3].wrapping_add(diff);
        RgbaColor([self.0[0], self.0[1], self.0[2], new_alpha])
    }

    pub fn apply_diff(&self, b1: u8) -> Self {
        let r = self.0[0].wrapping_add(b1 >> 4 & 0x03).wrapping_sub(2);
        let g = self.0[1].wrapping_add(b1 >> 2 & 0x03).wrapping_sub(2);
        let b = self.0[2].wrapping_add(b1 & 0x03).wrapping_sub(2);
        RgbaColor([r, g, b, self.0[3]])
    }

    pub fn apply_luma(&self, b1: u8, b2: u8) -> Self {
        let vg = (b1 & 0x3f).wrapping_sub(32);
        let vr = vg.wrapping_sub(8).wrapping_add(b2 >> 4 & 0x0f);
        let vb = vg.wrapping_sub(8).wrapping_add(b2 & 0x0f);

        let r = self.0[0].wrapping_add(vr);
        let g = self.0[1].wrapping_add(vg);
        let b = self.0[2].wrapping_add(vb);

        RgbaColor([r, g, b, self.0[3]])
    }
}

pub fn color_diff(diff: (u8, u8, u8)) -> Option<u8> {
    let r = diff.0.wrapping_add(2);
    let g = diff.1.wrapping_add(2);
    let b = diff.2.wrapping_add(2);

    match r | g | b {
        0x00..=0x03 => Some(Op::Diff as u8 | (r << 4) | (g << 2) | b),
        _ => None,
    }
}

pub fn luma_diff(diff: (u8, u8, u8)) -> Option<[u8; 2]> {
    let r = diff.0.wrapping_add(8).wrapping_sub(diff.1);
    let g = diff.1.wrapping_add(32);
    let b = diff.2.wrapping_add(8).wrapping_sub(diff.1);

    match (r | b, g) {
        (0x00..=0x0F, 0x00..=0x3F) => Some([Op::Luma as u8 | g, r << 4 | b]),
        _ => None,
    }
}

#[derive(IntoPrimitive, Copy, Clone, TryFromPrimitive, Debug)]
#[repr(u32)]
pub enum Compression {
    None = 0,
    Lz4 = 1,
}

#[derive(IntoPrimitive, Copy, Clone, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Channels {
    Gray = 1,
    GrayAlpha = 2,
    Rgb = 3,
    Rgba = 4,
}

#[derive(IntoPrimitive, Copy, Clone, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Colorspace {
    Srgb,
}
