use super::writer::Writer;
use crate::{
    types::{color_diff, luma_diff, Channels, Op, RgbaColor, CACHE_SIZE, END_OF_IMAGE},
    util::pixel_hash,
};
use lz4_flex::frame::FrameEncoder;
use std::io::{self, BufWriter, Read, Write};

// PixelEncoder is a stream encoder that encodes pixels one by one
// - Writer is a wrapper around the underlying writer that can be either a lz4 encoder or a regular writer
// - C is the number of channels in the image
pub struct PixelEncoder<W: Write, const C: usize> {
    writer: Writer<W>,
    runlength: u8,    // if runlength > 0 then we are in runlength encoding mode
    pixels_in: usize, // pixels encoded so far
    pixels_count: usize,

    cache: [RgbaColor; CACHE_SIZE],
    prev_pixel: RgbaColor,

    buffer: Vec<u8>,
}

impl<W: Write, const C: usize> PixelEncoder<W, C> {
    pub fn new(writer: Writer<W>, pixels_count: usize) -> Self {
        Self {
            writer,
            cache: [RgbaColor([0, 0, 0, 0]); CACHE_SIZE],
            runlength: 0,
            pixels_in: 0,
            pixels_count,
            prev_pixel: RgbaColor([0, 0, 0, 0]),

            buffer: Vec::with_capacity(8),
        }
    }

    pub fn new_lz4(writer: W, pixels_count: usize) -> Self {
        Self::new(
            Writer::Lz4Encoder(Box::new(FrameEncoder::new(writer))),
            pixels_count,
        )
    }

    pub fn new_uncompressed(writer: W, pixels_count: usize) -> Self {
        Self::new(
            Writer::UncompressedEncoder(BufWriter::new(writer)),
            pixels_count,
        )
    }

    fn encode_runlength(&mut self, curr_pixel: RgbaColor) -> u8 {
        let res = if self.runlength == 1 {
            self.runlength = 0;
            pixel_hash(curr_pixel)
        } else {
            0xc0 | (self.runlength - 1)
        };
        self.runlength = 0;
        res
    }

    #[inline]
    fn encode_pixel(
        &mut self,
        curr_pixel: RgbaColor,
        prev_pixel: RgbaColor,
    ) -> std::io::Result<()> {
        if C < Channels::Rgba as u8 as usize {
            // alpha channel should be 255 for all pixels in RGB images
            assert_eq!(curr_pixel.0[3], 255)
        };

        self.pixels_in += 1;
        let mut curr_pixel = curr_pixel;

        // runlength encode
        if curr_pixel == prev_pixel {
            self.runlength += 1;
            // skip further encoding since runlength is increasing and we have not reached the end of the image
            if self.runlength < (CACHE_SIZE as u8 - 2) && self.pixels_in < self.pixels_count {
                return Ok(());
            }
        }
        // previous pixel was not the same or we reached the end of the image so we need to encode the runlength
        if self.runlength > 0 {
            println!("adding runlength: {}", self.runlength);
            let res = self.encode_runlength(curr_pixel);
            self.cache_pixel(&mut curr_pixel);
            self.writer.write_all(&[res])?;
            return Ok(());
        }

        // index encoding
        let hash = pixel_hash(curr_pixel);
        if self.cache[hash as usize] == curr_pixel {
            self.cache_pixel(&mut curr_pixel);
            self.writer.write_all(&[u8::from(Op::Index) | hash])?;
            return Ok(());
        }

        // RGBA encoding (whenever alpha channel changes)
        if C > Channels::Rgb as u8 as usize && curr_pixel.0[3] != prev_pixel.0[3] {
            println!("adding rgba");
            self.cache_pixel(&mut curr_pixel);
            self.writer.write_all(&[Op::Rgba as u8])?;
            self.writer.write_all(&curr_pixel.0)?;
            return Ok(());
        }

        // Difference between current and previous pixel
        let diff = curr_pixel.diff(&prev_pixel);

        // Diff encoding
        if let Some(diff) = color_diff(diff) {
            self.cache_pixel(&mut curr_pixel);
            self.writer.write_all(&[diff])?;
            return Ok(());
        }

        // Luma encoding
        if let Some(luma) = luma_diff(diff) {
            self.cache_pixel(&mut curr_pixel);
            self.writer.write_all(&luma)?;
            return Ok(());
        }

        // RGB encoding
        let RgbaColor([r, g, b, _]) = curr_pixel;
        self.cache_pixel(&mut curr_pixel);
        self.writer.write_all(&[Op::Rgb as u8, r, g, b])?;
        Ok(())
    }

    #[inline]
    fn cache_pixel(&mut self, curr_pixel: &mut RgbaColor) {
        let hash = pixel_hash(*curr_pixel);
        self.cache[hash as usize] = *curr_pixel;
    }

    // flushes the remaining pixels in the cache and writes the end of image marker, automatically called after N pixels are encoded
    pub fn finish(&mut self) -> std::io::Result<()> {
        self.writer.write_all(&END_OF_IMAGE)
    }

    // take a reader and encode it pixel by pixel
    pub fn encode<R: Read>(&mut self, mut reader: R) -> std::io::Result<u64> {
        io::copy(&mut reader, self)
    }
}

impl<W: Write, const C: usize> Write for PixelEncoder<W, C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf_len = buf.len();
        let mut total_bytes_written = 0;

        for byte in buf {
            self.buffer.push(*byte);
            if self.buffer.len() == C {
                let mut curr_pixel = RgbaColor([0, 0, 0, 255]);
                curr_pixel.0[..C].copy_from_slice(&self.buffer);
                self.encode_pixel(curr_pixel, self.prev_pixel)?;
                self.prev_pixel = curr_pixel;
                self.buffer.clear();
                total_bytes_written += C;
            }
        }

        if self.pixels_in == self.pixels_count {
            println!("writing end");
            self.finish()?;
        }

        println!("total bytes written: {}", total_bytes_written);
        Ok(buf_len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()?;

        if !self.buffer.is_empty() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "buffer not empty",
            ))
        } else {
            Ok(())
        }
    }
}
