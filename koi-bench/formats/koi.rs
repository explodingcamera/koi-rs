use std::io::Result;

use koi::{file::FileHeader, types::Compression};

use super::ImageFormat;

#[derive(Debug)]
pub struct Koi<const C: usize> {}

impl<const C: usize> ImageFormat for Koi<C> {
    fn encode(&mut self, data: &[u8], out: &mut Vec<u8>, dimensions: (u32, u32)) -> Result<()> {
        Ok(koi::encode::<_, _, C, 0>(
            FileHeader::new(
                None,
                dimensions.0,
                dimensions.1,
                (C as u8).try_into().expect("Koi: Invalid channel count"),
                Compression::None,
            ),
            data,
            out,
        )?)
    }

    fn decode(&mut self, data: &[u8], out: &mut Vec<u8>, _dimensions: (u32, u32)) -> Result<()> {
        Ok(koi::decode::<_, _, C>(data, out).map(|_| ())?)
    }
}

#[derive(Debug)]
pub struct KoiLz4<const C: usize> {}

impl<const C: usize> KoiLz4<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize> ImageFormat for KoiLz4<C> {
    fn encode(&mut self, data: &[u8], out: &mut Vec<u8>, dimensions: (u32, u32)) -> Result<()> {
        Ok(koi::encode::<_, _, C, 1>(
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

    fn decode(&mut self, data: &[u8], out: &mut Vec<u8>, _dimensions: (u32, u32)) -> Result<()> {
        Ok(koi::decode::<_, _, C>(data, out).map(|_| ())?)
    }
}
