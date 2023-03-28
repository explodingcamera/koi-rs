use std::io::Result;

use super::ImageFormat;

#[derive(Debug)]
pub struct Png<const C: usize> {}

impl<const C: usize> Png<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize, W: std::io::Write, R: std::io::Read> ImageFormat<W, R> for Png<C> {
    fn encode(&mut self, data: R, out: W, dimensions: (u32, u32)) -> Result<()> {
        encode::<_, _, C>(data, out, dimensions)
    }

    fn decode(&mut self, data: R, out: W, _dimensions: (u32, u32)) -> Result<()> {
        decode::<_, _, C>(data, out)
    }
}

fn encode<W: std::io::Write, R: std::io::Read, const C: usize>(
    mut reader: R,
    writer: W,
    dimensions: (u32, u32),
) -> Result<()> {
    let mut encoder = png::Encoder::new(writer, dimensions.0, dimensions.1);
    let mut data = Vec::with_capacity(((dimensions.0 * dimensions.1) / 2) as usize * C);
    reader.read_to_end(&mut data)?;
    encoder.set_compression(png::Compression::Default);

    if C == 3 {
        encoder.set_color(png::ColorType::Rgb);
    } else {
        encoder.set_color(png::ColorType::Rgba);
    }

    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(&data)?;

    Ok(())
}

fn decode<W: std::io::Write, R: std::io::Read, const C: usize>(
    reader: R,
    mut writer: W,
) -> Result<()> {
    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf)?;

    writer.write_all(&buf)?;
    Ok(())
}
