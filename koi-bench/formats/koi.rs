use koi::{file::FileHeader, types::Compression};
use std::io::Result;

use super::ImageFormat;

#[derive(Debug)]
pub struct Koi<const C: usize> {}

impl<const C: usize> ImageFormat for Koi<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let header = FileHeader::new(
            1,
            None,
            dimensions.0 as u64,
            dimensions.1 as u64,
            (C as u8).try_into().expect("Koi: Invalid channel count"),
            Compression::Lz4,
            None,
            None,
        );

        let data = koi::encoder::block::encode_to_vec::<C>(
            data,
            header,
            koi::encoder::block::CompressionLevel::Lz4Hc(4),
        )?;
        Ok(data)
    }

    fn decode(&mut self, data: &[u8], _dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let res = koi::decoder::block::decode_to_vec::<C>(data)?;
        Ok(res.data)
    }
}

#[derive(Debug)]
pub struct KoiFast<const C: usize> {}

impl<const C: usize> ImageFormat for KoiFast<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let header = FileHeader::new(
            1,
            None,
            dimensions.0 as u64,
            dimensions.1 as u64,
            (C as u8).try_into().expect("Koi: Invalid channel count"),
            Compression::Lz4,
            None,
            None,
        );

        let data = koi::encoder::block::encode_to_vec::<C>(
            data,
            header,
            koi::encoder::block::CompressionLevel::Lz4Flex,
        )?;

        Ok(data)
    }

    fn decode(&mut self, data: &[u8], _dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let res = koi::decoder::block::decode_to_vec::<C>(data)?;
        Ok(res.data)
    }
}
