use super::writer::Writer;
use crate::{
    file::FileHeader,
    types::{Channels, Op, RgbaColor, CACHE_SIZE},
    util::pixel_hash,
};
use lz4_flex::frame::FrameEncoder;
use std::io::{BufWriter, Write};

pub struct PixelEncoder<W: Write, const C: usize> {
    header: FileHeader,
    write_encoder: Writer<W>,

    runlength: u8,
    pixel_pos: u8,

    pixels_in: usize,
    pixels_count: usize, // total pixels in image
    cache: [RgbaColor; 64],
}

enum EncodedPixel {
    Skip,
    Single([u8; 1]),
    Quad([u8; 4]),
    Full([u8; 5]),
}

impl<W: Write, const C: usize> PixelEncoder<W, C> {
    pub fn new(writer: Writer<W>, header: FileHeader) -> Self {
        Self {
            write_encoder: writer,
            cache: [RgbaColor([0, 0, 0, 0]); CACHE_SIZE],
            runlength: 0,
            pixels_in: 0,
            pixels_count: 0,
            pixel_pos: 0,
            header,
        }
    }

    pub fn new_lz4(writer: W, header: FileHeader) -> Self {
        Self::new(
            Writer::Lz4Encoder(Box::new(FrameEncoder::new(writer))),
            header,
        )
    }

    pub fn new_uncompressed(writer: W, header: FileHeader) -> Self {
        Self::new(Writer::UncompressedEncoder(BufWriter::new(writer)), header)
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

    fn encode_pixel(
        &mut self,
        curr_pixel: RgbaColor,
        prev_pixel: RgbaColor,
    ) -> std::io::Result<EncodedPixel> {
        if C == 4 {
            assert_eq!(curr_pixel.0[3], 255);
        };
        let mut curr_pixel = curr_pixel;

        // runlength encoding
        {
            if curr_pixel == prev_pixel {
                self.runlength += 1;
                if self.runlength < (CACHE_SIZE as u8 - 2) && self.pixels_in < self.pixels_count {
                    return Ok(EncodedPixel::Skip);
                }
            }
            if self.runlength > 0 {
                // finish runlength
                let res = self.encode_runlength(curr_pixel);
                self.cache_pixel(&mut curr_pixel);
                return Ok(EncodedPixel::Single([res]));
            }
        }

        // index encoding
        {
            let hash = pixel_hash(curr_pixel);
            if self.cache[hash as usize] == curr_pixel {
                self.cache_pixel(&mut curr_pixel);
                return Ok(EncodedPixel::Single([u8::from(Op::Index) | hash]));
            }
        }

        // RGBA encoding
        {
            if C > Channels::Rgb as u8 as usize {
                let RgbaColor([r, g, b, a]) = curr_pixel;
                self.cache_pixel(&mut curr_pixel);
                return Ok(EncodedPixel::Full([Op::Rgba.into(), r, g, b, a]));
            }
        }

        {
            let RgbaColor([r, g, b, a]) = curr_pixel;
            let RgbaColor([pr, pg, pb, pa]) = prev_pixel;

            let diff = RgbaColor([
                r.wrapping_sub(pr),
                g.wrapping_sub(pg),
                b.wrapping_sub(pb),
                a.wrapping_sub(pa),
            ]);

            // Diff encoding
            {
                if diff != RgbaColor([0, 0, 0, 0]) {
                    self.cache_pixel(&mut curr_pixel);
                    return Ok(EncodedPixel::Full([
                        Op::Diff.into(),
                        diff.0[0],
                        diff.0[1],
                        diff.0[2],
                        diff.0[3],
                    ]));
                }
            }

            // // Luma encoding
            // {
            //     if
            // }
        }

        // RGB encoding
        {
            let RgbaColor([r, g, b, _]) = curr_pixel;
            self.cache_pixel(&mut curr_pixel);
            return Ok(EncodedPixel::Quad([Op::Rgb.into(), r, g, b]));
        }

        // Gray encoding
        // {
        //     let RgbaColor([r, _, _, _]) = self.curr_pixel;
        //     self.finish_encode_byte();
        //     return Ok(Some(&[Op::Gray.into(), r]));
        // }

        Ok(EncodedPixel::Skip)
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
