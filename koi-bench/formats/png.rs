use std::io::Result;

use super::ImageFormat;

#[derive(Debug)]
pub struct Png<const C: usize> {}

impl<const C: usize> Png<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize> ImageFormat for Png<C> {
    fn encode(&mut self, data: &[u8], out: &mut [u8], dimensions: (u32, u32)) -> Result<()> {
        let mut encoder = png::Encoder::new(out, dimensions.0, dimensions.1);

        if C == 3 {
            encoder.set_color(png::ColorType::Rgb);
        } else {
            encoder.set_color(png::ColorType::Rgba);
        }

        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(data)?;
        writer.finish()?;

        Ok(())
    }

    fn decode(&mut self, data: &[u8], out: &mut [u8], _dimensions: (u32, u32)) -> Result<()> {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        reader.next_frame(out)?;
        Ok(())
    }
}
