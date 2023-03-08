use super::writer::Writer;
use crate::{
    file::FileHeader,
    types::{Channels, Op, RgbaColor, CACHE_SIZE},
    util::pixel_hash,
};
use lz4_flex::frame::FrameEncoder;
use std::{
    io::{BufWriter, Write},
    ops::Sub,
};

pub struct PixelEncoder<W: Write, const C: usize> {
    write_encoder: Writer<W>,

    runlength: u8,
    pixel_pos: u8,

    pixels_in: usize,
    pixels_count: usize, // total pixels in image
    cache: [RgbaColor; 64],
}

impl<W: Write, const C: usize> PixelEncoder<W, C> {
    pub fn new(writer: Writer<W>) -> Self {
        Self {
            write_encoder: writer,
            cache: [RgbaColor([0, 0, 0, 0]); CACHE_SIZE],
            runlength: 0,
            pixels_in: 0,
            pixels_count: 0,
            pixel_pos: 0,
        }
    }

    pub fn new_lz4(writer: W) -> Self {
        Self::new(Writer::Lz4Encoder(Box::new(FrameEncoder::new(writer))))
    }

    pub fn new_uncompressed(writer: W) -> Self {
        Self::new(Writer::UncompressedEncoder(BufWriter::new(writer)))
    }

    fn encode_runlength(&mut self, curr_pixel: RgbaColor) -> u8 {
        if self.runlength == 1 {
            self.runlength = 0;
            pixel_hash(curr_pixel)
        } else {
            self.runlength = 0;
            0xc0 | (self.runlength - 1)
        }
    }

    #[inline]
    pub fn encode_pixel(
        &mut self,
        curr_pixel: RgbaColor,
        prev_pixel: RgbaColor,
        writer: &mut W,
    ) -> std::io::Result<()> {
        if C == 4 {
            assert_eq!(curr_pixel.0[3], 255);
        };
        let mut curr_pixel = curr_pixel;

        // runlength encoding
        {
            if curr_pixel == prev_pixel {
                self.runlength += 1;
                if self.runlength < (CACHE_SIZE as u8 - 2) && self.pixels_in < self.pixels_count {
                    return Ok(());
                }
            }
            if self.runlength > 0 {
                // finish runlength
                let res = self.encode_runlength(curr_pixel);
                self.cache_pixel(&mut curr_pixel);
                writer.write_all(&[res])?;
                return Ok(());
            }
        }

        // index encoding
        {
            let hash = pixel_hash(curr_pixel);
            if self.cache[hash as usize] == curr_pixel {
                self.cache_pixel(&mut curr_pixel);
                writer.write_all(&[u8::from(Op::Index) | hash])?;
                return Ok(());
            }
        }

        // RGBA encoding
        {
            if C > Channels::Rgb as u8 as usize {
                let RgbaColor([r, g, b, a]) = curr_pixel;
                self.cache_pixel(&mut curr_pixel);
                writer.write_all(&[Op::Rgba as u8, r, g, b, a])?;
                return Ok(());
            }
        }

        {
            let diff = curr_pixel.diff_rgb(prev_pixel);

            // Diff encoding
            {
                if diff[0].abs() <= 1 && diff[1].abs() <= 1 && diff[2].abs() <= 1 {
                    self.cache_pixel(&mut curr_pixel);
                    writer.write_all(&[Op::Diff as u8
                        | (((diff[0] + 2) << 4) as u8)
                        | (((diff[1] + 2) << 2) as u8)
                        | ((diff[2] + 2) as u8)])?;
                    return Ok(());
                }
            }

            // Luma encoding
            {
                let diff_rg: i8 = diff[0].sub(diff[1]);
                let diff_bg: i8 = diff[2].sub(diff[1]);

                if (-8..=7).contains(&diff_rg)
                    && (-8..=7).contains(&diff_bg)
                    && !(-31..=32).contains(&diff[1])
                {
                    self.cache_pixel(&mut curr_pixel);
                    writer.write_all(&[
                        Op::Luma as u8 | diff[1] as u8,
                        ((diff_rg + 8) << 4) as u8 | ((diff_bg + 8) as u8),
                    ])?;

                    return Ok(());
                }
            }
        }

        // RGB encoding
        let RgbaColor([r, g, b, _]) = curr_pixel;
        self.cache_pixel(&mut curr_pixel);
        writer.write_all(&[Op::Rgb as u8, r, g, b])?;

        Ok(())
    }

    fn cache_pixel(&mut self, curr_pixel: &mut RgbaColor) {
        let hash = pixel_hash(*curr_pixel);
        self.cache[hash as usize] = *curr_pixel;
    }
}

impl<W: Write, const C: usize> Write for PixelEncoder<W, C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.write_encoder.flush()
    }
}
