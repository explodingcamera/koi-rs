use super::ImageFormat;
use std::io::Result;

#[derive(Debug)]
pub struct Qoi<const C: usize> {}

impl<const C: usize> Qoi<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize> ImageFormat for Qoi<C> {
    fn encode(&mut self, data: &[u8], out: &mut Vec<u8>, dimensions: (u32, u32)) -> Result<()> {
        *out = qoi::encode_to_vec(data, dimensions.0, dimensions.1)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(())
    }

    fn decode(&mut self, data: &[u8], out: &mut Vec<u8>, _dimensions: (u32, u32)) -> Result<()> {
        (_, *out) = qoi::decode_to_vec(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(())
    }
}
