use koi::{file::FileHeader, types::Compression};

use super::ImageFormat;

#[derive(Debug)]
pub struct Koi {}

impl Koi {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageFormat for Koi {
    fn encode<const C: usize>(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32),
    ) -> Result<(), ()> {
        koi::encode::<_, _, C>(
            FileHeader::new(
                None,
                dimensions.0,
                dimensions.1,
                (C as u8).try_into().expect("Koi: Invalid channel count"),
                Compression::Lz4,
            ),
            data,
            out,
        )
        .map(|_| ())
        .map_err(|_| ())
    }

    fn decode<const C: usize>(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32),
    ) -> Result<(), ()> {
        koi::decode::<_, _, C>(data, out)
            .map(|_| ())
            .map_err(|_| ())
    }
}
