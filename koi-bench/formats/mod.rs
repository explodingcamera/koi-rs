pub mod koi;
pub mod png;
pub mod pngfast;
pub mod qoi;
pub mod webp;

use strum_macros::{Display, EnumIter};

#[derive(Debug, Display, EnumIter, PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Copy)]
pub enum ImageFormatType {
    Png,
    PngFast,
    Koi,
    // QOI,
    // WEBP,
}

impl ImageFormatType {
    pub fn get_impl<const C: usize>(&self) -> Box<dyn ImageFormat> {
        match self {
            ImageFormatType::Png => Box::new(png::Png::<C>::new()),
            ImageFormatType::PngFast => Box::new(pngfast::PngFast::<C>::new()),
            ImageFormatType::Koi => Box::new(koi::Koi::<C>::new()),
            // ImageFormatType::QOI => Box::new(qoi::Qoi::<C>::new()),
            // ImageFormatType::WEBP => Box::new(webp::Webp::<C>::new()),
        }
    }

    pub fn get_impl_dyn(&self, channels: usize) -> Box<dyn ImageFormat> {
        match channels {
            3 => self.get_impl::<3>(),
            4 => self.get_impl::<4>(),
            _ => panic!("Unsupported number of channels"),
        }
    }

    // pub fn encode<W: std::io::Write, R: std::io::Read>(
    //     &self,
    //     data: R,
    //     out: W,
    //     dimensions: (u32, u32),
    //     channels: usize,
    // ) -> std::io::Result<()> {
    //     self.get_impl_dyn::<W, R>(channels)
    //         .encode(data, out, dimensions)
    // }

    // pub fn decode<W: std::io::Write, R: std::io::Read>(
    //     &self,
    //     data: R,
    //     out: W,
    //     dimensions: (u32, u32),
    //     channels: usize,
    // ) -> std::io::Result<()> {
    //     self.get_impl_dyn::<W, R>(channels)
    //         .decode(data, out, dimensions)
    // }
}

pub trait ImageFormat {
    fn encode(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32), // (width, height)
    ) -> std::io::Result<()>;

    fn decode(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32), // (width, height)
    ) -> std::io::Result<()>;
}
