pub mod koi;
pub mod png;
pub mod qoi;
pub mod webp;

use ::koi::util::cold;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Display, EnumIter, PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Copy)]
pub enum ImageFormatType {
    Png,
    PngFast,
    // Koi,
    Koi0Lz4,
    Koi2,
    Qoi,
}

impl ImageFormatType {
    pub fn get_impl<const C: usize>(&self) -> Box<dyn ImageFormat> {
        match self {
            ImageFormatType::Png => Box::new(png::Png::<C> {}),
            ImageFormatType::PngFast => Box::new(png::PngFast::<C> {}),
            // ImageFormatType::Koi => Box::new(koi::Koi::<C>::new()),
            ImageFormatType::Koi0Lz4 => Box::new(koi::Koi0Lz4::<C> {}),
            ImageFormatType::Koi2 => Box::new(koi::Koi2::<C> {}),
            ImageFormatType::Qoi => Box::new(qoi::Qoi::<C>::new()),
        }
    }

    pub fn get_impl_dyn(&self, channels: usize) -> Box<dyn ImageFormat> {
        match channels {
            3 => self.get_impl::<3>(),
            4 => self.get_impl::<4>(),
            _ => {
                cold();
                panic!("Unsupported number of channels")
            }
        }
    }
}

pub trait ImageFormat {
    fn encode(
        &mut self,
        data: &[u8],
        dimensions: (u32, u32), // (width, height)
    ) -> std::io::Result<Vec<u8>>;

    fn decode(
        &mut self,
        data: &[u8],
        dimensions: (u32, u32), // (width, height)
    ) -> std::io::Result<Vec<u8>>;
}
