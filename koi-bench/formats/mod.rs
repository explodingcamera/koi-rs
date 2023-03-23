use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

pub trait ImageFormat {
    fn encode<const C: usize>(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32), // (width, height)
    ) -> Result<(), ()>;

    fn decode<const C: usize>(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32), // (width, height)
    ) -> Result<(), ()>;
}

#[derive(Debug, Display, EnumIter, PartialEq, Eq, Hash)]
pub enum ImageFormatType {
    PNG,
    QOI,
    KOI,
    WEBP,
}

pub mod koi;
pub mod png;
pub mod qoi;
pub mod webp;
