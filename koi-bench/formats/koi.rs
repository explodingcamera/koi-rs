use std::io::Result;

use koi::{
    file::FileHeader,
    types::{Compression, MAGIC},
};

use super::ImageFormat;

#[derive(Debug)]
pub struct Koi<const C: usize> {}

impl<const C: usize> ImageFormat for Koi<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = vec![0; (data.len() * 3) / 2];

        koi::encode::<_, _, C, 0>(
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
        let mut out = vec![0; (data.len() * 3) / 2];
        koi::decode::<_, _, C>(data, &mut out).map(|_| ())?;
        Ok(out)
    }
}

#[derive(Debug)]
pub struct KoiLz4<const C: usize> {}

impl<const C: usize> ImageFormat for KoiLz4<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = vec![0; (data.len() * 3) / 2];

        koi::encode::<_, _, C, 1>(
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

        println!("magic: {:?}", &out[..8]);

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
        let mut out = vec![0; (data.len() * 3) / 2];

        let header = FileHeader::new(
            None,
            dimensions.0,
            dimensions.1,
            (C as u8).try_into().expect("Koi: Invalid channel count"),
            Compression::Lz4b,
        );

        let header_size = header.write_to_buf(&mut out)?;
        koi::encoder::block::encode::<C>(data, &mut out[header_size..])?;

        Ok(out)
    }

    fn decode(&mut self, data: &[u8], _dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let mut out = vec![0; (data.len() * 3) / 2];
        koi::decode::<_, _, C>(data, &mut out).map(|_| ())?;
        Ok(out)
    }
}
