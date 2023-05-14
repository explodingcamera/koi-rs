use crate::util::unlikely;

// magic number to identify koi files
pub const MAGIC: [u8; 8] = *b"KOI \xF0\x9F\x99\x82";
pub const MASK: u8 = 0xC0;
pub const END_OF_IMAGE: [u8; 4] = *b"\xF0\x9F\x99\x82";

// pub const OP_INDEX: u8 = 0x00;
// pub const OP_INDEX_END: u8 = 0x3F;
// pub const OP_DIFF: u8 = 0x40;
// pub const OP_DIFF_END: u8 = 0x40 | 0x3F;
// pub const OP_LUMA: u8 = 0x80;
// pub const OP_LUMA_END: u8 = 0x80 | 0x3F;

pub const OP_DIFF: u8 = 0x00;
pub const OP_DIFF_END: u8 = 0x00 | 0x3F;

pub const OP_LUMA: u8 = 0x40;
pub const OP_LUMA_END: u8 = 0x40 | 0x3F;

pub const OP_DIFF_ALPHA: u8 = 0xC0;
pub const OP_DIFF_ALPHA_END: u8 = 0xC0 | 0x3b; // we only have 59 possible values for diff alpha so we can use the color opcodes
pub const OP_GRAY: u8 = 0xfc;
pub const OP_GRAY_ALPHA: u8 = 0xfd;
pub const OP_RGB: u8 = 0xfe;
pub const OP_RGBA: u8 = 0xff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Op {
    // Index = OP_INDEX,
    Diff = OP_DIFF,
    Luma = OP_LUMA,
    DiffAlpha = OP_DIFF_ALPHA,
    Gray = OP_GRAY,
    GrayAlpha = OP_GRAY_ALPHA,
    Rgb = OP_RGB,
    Rgba = OP_RGBA,
}

// impl Op => u8
impl From<Op> for u8 {
    fn from(op: Op) -> Self {
        op as u8
    }
}

impl From<u8> for Op {
    fn from(op: u8) -> Self {
        match op {
            // OP_INDEX..=OP_INDEX_END => Op::Index,
            OP_DIFF..=OP_DIFF_END => Op::Diff,
            OP_LUMA..=OP_LUMA_END => Op::Luma,
            OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => Op::DiffAlpha,
            OP_GRAY => Op::Gray,
            OP_GRAY_ALPHA => Op::GrayAlpha,
            OP_RGB => Op::Rgb,
            OP_RGBA => Op::Rgba,
            _ => panic!("Invalid opcode {}", op),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel<const C: usize> {
    pub data: [u8; C],
}

impl<const C: usize> PartialEq for Pixel<C> {
    fn eq(&self, other: &Self) -> bool {
        self.data[..] == other.data[..]
    }
}

impl<const C: usize> Default for Pixel<C> {
    fn default() -> Self {
        Pixel { data: [255; C] }
    }
}

impl<const C: usize, const C2: usize> From<[u8; C2]> for Pixel<C> {
    fn from(data: [u8; C2]) -> Self {
        let (r, g, b, a) = match C2 {
            1 => (data[0], 0, 0, 255),
            2 => (data[0], 0, 0, data[1]),
            3 => (data[0], data[1], data[2], 255),
            4 => (data[0], data[1], data[2], data[3]),
            _ => unreachable!(),
        };

        Pixel {
            data: [r, g, b, a][..C].try_into().unwrap(),
        }
    }
}

impl<const C: usize> From<&[u8]> for Pixel<C> {
    fn from(bytes: &[u8]) -> Self {
        let mut px = Pixel::default();
        px.data[..C].copy_from_slice(&bytes[..C]);
        px
    }
}

impl<const C: usize> Pixel<C> {
    pub fn rgb(&self) -> [u8; 3] {
        [self.r(), self.g(), self.b()]
    }

    pub fn rgba(&self) -> [u8; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }

    pub fn r(&self) -> u8 {
        self.data[0]
    }

    pub fn g(&self) -> u8 {
        match C {
            3 | 4 => self.data[1],
            1 | 2 => self.data[0],
            _ => unreachable!(),
        }
    }

    pub fn b(&self) -> u8 {
        match C {
            3 | 4 => self.data[2],
            1 | 2 => self.data[0],
            _ => unreachable!(),
        }
    }

    pub fn a(&self) -> u8 {
        match C {
            4 => self.data[3],
            2 => self.data[1],
            1 | 3 => 0xff,
            _ => unreachable!(),
        }
    }

    pub fn from_grayscale(gray: u8) -> Self {
        match C {
            3 => [gray, gray, gray, 0xff].into(),
            4 => [gray, gray, gray, 0xff].into(),
            2 => [gray, 0xff].into(),
            1 => [gray].into(),
            _ => unreachable!(),
        }
    }

    pub fn is_gray(&self) -> bool {
        match C {
            4 | 3 => unlikely(self.data[0] == self.data[1] && self.data[1] == self.data[2]),
            2 | 1 => true,
            _ => unreachable!(),
        }
    }

    pub fn diff(&self, other: &Self) -> Diff {
        let r = self.r().wrapping_sub(other.r());
        let g = self.g().wrapping_sub(other.g());
        let b = self.b().wrapping_sub(other.b());
        Diff(r, g, b)
    }

    pub fn alpha_diff(&self, other: &Self) -> Option<u8> {
        let a1 = self.a();
        let a2 = other.a();
        let diff = a2.wrapping_sub(a1).wrapping_add(0x1e);

        match diff {
            0x00..=0x3b => Some(Op::DiffAlpha as u8 | diff),
            _ => None,
        }
    }

    pub fn apply_alpha_diff(&self, op: u8) -> Self {
        let diff = (op & !(Op::DiffAlpha as u8)).wrapping_sub(0x1e);
        let new_alpha = self.a().wrapping_add(diff);
        [self.r(), self.g(), self.b(), new_alpha].into()
    }

    pub fn apply_diff(&self, b1: u8) -> Self {
        let r = self.r().wrapping_add(b1 >> 4 & 0x03).wrapping_sub(2);
        let g = self.g().wrapping_add(b1 >> 2 & 0x03).wrapping_sub(2);
        let b = self.b().wrapping_add(b1 & 0x03).wrapping_sub(2);

        [r, g, b].into()
    }

    pub fn apply_luma(&self, b1: u8, b2: u8) -> Self {
        let vg = (b1 & 0x3f).wrapping_sub(32);
        let vr = ((b2 >> 4) & 0x0f).wrapping_sub(8).wrapping_add(vg);
        let vb = (b2 & 0x0f).wrapping_sub(8).wrapping_add(vg);

        let r = self.r().wrapping_add(vr);
        let g = self.g().wrapping_add(vg);
        let b = self.b().wrapping_add(vb);

        [r, g, b, self.a()].into()
    }

    // #[inline]
    // pub fn hash(self) -> u8 {
    //     // index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 64
    //     let (r, g, b, a) = match C {
    //         4 => (self.data[0], self.data[1], self.data[2], self.data[3]),
    //         3 => (self.data[0], self.data[1], self.data[2], 0xff),
    //         2 => (self.data[0], self.data[0], self.data[0], self.data[1]),
    //         1 => (self.data[0], self.data[0], self.data[0], 0xff),
    //         _ => unreachable!(),
    //     };

    //     ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % CACHE_SIZE as u32) as u8
    // }
}

pub struct Diff(u8, u8, u8);
impl Diff {
    pub fn color(&self) -> Option<u8> {
        let r = self.0.wrapping_add(2);
        let g = self.1.wrapping_add(2);
        let b = self.2.wrapping_add(2);

        match r | g | b {
            0x00..=0x03 => Some(Op::Diff as u8 | (r << 4) | (g << 2) | b),
            _ => None,
        }
    }

    pub fn luma(&self) -> Option<[u8; 2]> {
        let r = self.0.wrapping_add(8).wrapping_sub(self.1);
        let g = self.1.wrapping_add(32);
        let b = self.2.wrapping_add(8).wrapping_sub(self.1);

        match (r | b, g) {
            (0x00..=0x0F, 0x00..=0x3F) => Some([Op::Luma as u8 | g, r << 4 | b]),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum Compression {
    None = 0,
    Lz4 = 1,
}

impl TryFrom<u8> for Compression {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Compression::None),
            1 => Ok(Compression::Lz4),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Channels {
    Gray = 1,
    GrayAlpha = 2,
    Rgb = 3,
    Rgba = 4,
}

impl TryFrom<u8> for Channels {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Channels::Gray),
            2 => Ok(Channels::GrayAlpha),
            3 => Ok(Channels::Rgb),
            4 => Ok(Channels::Rgba),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Colorspace {
    Srgb,
}
