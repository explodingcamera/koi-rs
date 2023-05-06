use std::io::Result;

use super::ImageFormat;

#[derive(Debug)]
pub struct Png<const C: usize> {}

impl<const C: usize> ImageFormat for Png<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        encode_png::<C>(png::Compression::Default, data, dimensions)
    }

    fn decode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        let mut out = vec![0; reader.output_buffer_size()];

        let info = reader.next_frame(&mut out)?;

        if info.width != dimensions.0 || info.height != dimensions.1 {
            panic!("PngFast: Invalid dimensions");
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub struct PngFast<const C: usize> {}

impl<const C: usize> ImageFormat for PngFast<C> {
    fn encode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        encode_png::<C>(png::Compression::Fast, data, dimensions)
    }

    fn decode(&mut self, data: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>> {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        let mut out = vec![0; reader.output_buffer_size()];

        let info = reader.next_frame(&mut out)?;

        if info.width != dimensions.0 || info.height != dimensions.1 {
            panic!("PngFast: Invalid dimensions");
        }

        Ok(out)
    }
}

fn encode_png<const C: usize>(
    compression: png::Compression,
    data: &[u8],
    dimensions: (u32, u32),
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, dimensions.0, dimensions.1);
    encoder.set_compression(compression);

    if C == 3 {
        encoder.set_color(png::ColorType::Rgb);
    } else {
        encoder.set_color(png::ColorType::Rgba);
    }

    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    writer.finish()?;

    Ok(out)
}
