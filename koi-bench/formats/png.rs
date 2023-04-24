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
    fn encode(&mut self, data: &[u8], mut out: &mut Vec<u8>, dimensions: (u32, u32)) -> Result<()> {
        let mut encoder = png::Encoder::new(&mut out, dimensions.0, dimensions.1);
        encoder.set_compression(png::Compression::Default);

        if C == 3 {
            encoder.set_color(png::ColorType::Rgb);
        } else {
            encoder.set_color(png::ColorType::Rgba);
        }

        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(data)?;

        Ok(())
    }

    fn decode(
        &mut self,
        data: &[u8],
        mut _out: &mut Vec<u8>,
        _dimensions: (u32, u32),
    ) -> Result<()> {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        let mut out = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut out)?;
        *_out = out;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PngFast<const C: usize> {}

impl<const C: usize> PngFast<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize> ImageFormat for PngFast<C> {
    fn encode(&mut self, data: &[u8], mut out: &mut Vec<u8>, dimensions: (u32, u32)) -> Result<()> {
        let mut encoder = png::Encoder::new(&mut out, dimensions.0, dimensions.1);
        encoder.set_compression(png::Compression::Fast);

        if C == 3 {
            encoder.set_color(png::ColorType::Rgb);
        } else {
            encoder.set_color(png::ColorType::Rgba);
        }

        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(data)?;

        Ok(())
    }

    fn decode(
        &mut self,
        data: &[u8],
        mut _out: &mut Vec<u8>,
        _dimensions: (u32, u32),
    ) -> Result<()> {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        let mut out = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut out)?;
        *_out = out;
        Ok(())
    }
}
