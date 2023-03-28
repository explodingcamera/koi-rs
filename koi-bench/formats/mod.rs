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
    pub fn get_impl<const C: usize>(&self) -> Box<dyn ImageFormat> {
        match self {
            ImageFormatType::Png => Box::new(png::Png::<C>::new()),
            ImageFormatType::Koi => Box::new(koi::Koi::<C>::new()),
            // ImageFormatType::QOI => Box::new(qoi::Qoi::<C>::new()),
            // ImageFormatType::WEBP => Box::new(webp::Webp::<C>::new()),
        }
    }
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
