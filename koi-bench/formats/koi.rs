use std::io::Result;

use koi::{file::FileHeader, types::Compression};

use super::ImageFormat;

#[derive(Debug)]
pub struct Koi<const C: usize> {}

impl<const C: usize> Koi<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize, W: std::io::Write, R: std::io::Read> ImageFormat<W, R> for Koi<C> {
    fn encode(&mut self, data: R, out: W, dimensions: (u32, u32)) -> Result<()> {
        Ok(koi::encode::<_, _, C>(
            FileHeader::new(
                None,
                dimensions.0,
                dimensions.1,
                (C as u8).try_into().expect("Koi: Invalid channel count"),
                Compression::Lz4,
            ),
            data,
            out,
        )?)
    }

    fn decode(&mut self, data: R, out: W, _dimensions: (u32, u32)) -> Result<()> {
        Ok(koi::decode::<_, _, C>(data, out).map(|_| ())?)
    }
}
