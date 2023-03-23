use super::ImageFormat;

#[derive(Debug)]
pub struct Png {}

impl Png {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageFormat for Png {
    fn encode<const C: usize>(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32),
    ) -> Result<(), ()> {
        let mut encoder = png::Encoder::new(out, dimensions.0, dimensions.1);

        if C == 3 {
            encoder.set_color(png::ColorType::Rgb);
        } else {
            encoder.set_color(png::ColorType::Rgba);
        }

        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(data).unwrap();
        writer.finish().unwrap();

        Ok(())
    }

    fn decode<const C: usize>(
        &mut self,
        data: &[u8],
        out: &mut [u8],
        dimensions: (u32, u32),
    ) -> Result<(), ()> {
        let mut decoder = png::Decoder::new(data);
        decoder.set_transformations(png::Transformations::EXPAND);
        let mut reader = decoder.read_info().unwrap();
        reader.next_frame(out).unwrap();
        let info = reader.info().clone();
        Ok(())
    }
}
