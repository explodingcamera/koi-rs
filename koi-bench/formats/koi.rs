use koi::{file::FileHeader, types::Compression};
use std::io::Result;

use super::ImageFormat;

#[derive(Debug)]
pub struct Koi<const C: usize> {}

impl<const C: usize> ImageFormat for Koi<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(data.len() * 2);

        koi::encode::<_, _, C>(
            FileHeader::new(
                None,
                dimensions.0,
                dimensions.1,
                (C as u8).try_into().expect("Koi: Invalid channel count"),
                Compression::None,
            ),
            data,
            &mut out,
        )?;

        Ok(out)
    }

    fn decode(&mut self, data: &[u8], _dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(data.len() * 2);
        koi::decode::<_, _, C>(data, &mut out).map(|_| ())?;
        Ok(out)
    }
}

#[derive(Debug)]
pub struct KoiLz4<const C: usize> {}

impl<const C: usize> ImageFormat for KoiLz4<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(data.len() * 2);

        koi::encode::<_, _, C>(
            FileHeader::new(
                None,
                dimensions.0,
                dimensions.1,
                (C as u8).try_into().expect("Koi: Invalid channel count"),
                Compression::Lz4,
            ),
            data,
            &mut out,
        )?;

        Ok(out)
    }

    fn decode(&mut self, data: &[u8], _dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = vec![0; (data.len() * 3) / 2];
        koi::decode::<_, _, C>(data, &mut out).map(|_| ())?;
        Ok(out)
    }
}

#[derive(Debug)]
pub struct Koi2<const C: usize> {}

impl<const C: usize> ImageFormat for Koi2<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let header = FileHeader::new(
            None,
            dimensions.0,
            dimensions.1,
            (C as u8).try_into().expect("Koi: Invalid channel count"),
            Compression::Lz4b,
        );

        let data = koi::encoder::block::encode_to_vec::<C>(data, header)?;
        Ok(data)
    }

    fn decode(&mut self, data: &[u8], _dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = vec![0; data.len() * 2];
        koi::decode::<_, _, C>(data, &mut out).map(|_| ())?;
        Ok(out)
    }
}
