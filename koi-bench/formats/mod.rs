pub mod koi;
pub mod png;
pub mod qoi;
pub mod webp;

use strum_macros::{Display, EnumIter};

#[derive(Debug, Display, EnumIter, PartialEq, Eq, Hash)]
pub enum ImageFormatType {
    Png,
    Koi,
    // QOI,
    // WEBP,
}

impl ImageFormatType {
    pub fn get_impl<const C: usize, W: std::io::Write, R: std::io::Read>(
        &self,
    ) -> Box<dyn ImageFormat<W, R>> {
        match self {
            ImageFormatType::Png => Box::new(png::Png::<C>::new()),
            ImageFormatType::Koi => Box::new(koi::Koi::<C>::new()),
            // ImageFormatType::QOI => Box::new(qoi::Qoi::<C>::new()),
            // ImageFormatType::WEBP => Box::new(webp::Webp::<C>::new()),
        }
    }

    pub fn get_impl_dyn<W: std::io::Write, R: std::io::Read>(
        &self,
        channels: usize,
    ) -> Box<dyn ImageFormat<W, R>> {
        match channels {
            3 => self.get_impl::<3, _, _>(),
            4 => self.get_impl::<4, _, _>(),
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

pub trait ImageFormat<W: std::io::Write, R: std::io::Read> {
    fn encode(
        &mut self,
        data: R,
        out: W,
        dimensions: (u32, u32), // (width, height)
    ) -> std::io::Result<()>;

    fn decode(
        &mut self,
        data: R,
        out: W,
        dimensions: (u32, u32), // (width, height)
    ) -> std::io::Result<()>;
}
