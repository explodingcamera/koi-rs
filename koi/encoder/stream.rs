use super::writer::Writer;
use crate::types::{Channels, Op, Pixel, END_OF_IMAGE};
use lz4_flex::frame::FrameEncoder;
use std::io::{self, Read, Write};

// PixelEncoder is a stream encoder that encodes pixels one by one
// - Writer is a wrapper around the underlying writer that can be either a lz4 encoder or a regular writer
// - C is the number of channels in the image
pub struct PixelEncoder<W: Write, const C: usize> {
    writer: Writer<W>,
    pixels_in: usize, // pixels encoded so far
    pixels_count: usize,
    prev_pixel: Pixel<C>,

    remainder: smallvec::SmallVec<[u8; 3]>,
}

impl<W: Write, const C: usize> PixelEncoder<W, C> {
    pub fn new(writer: Writer<W>, pixels_count: usize) -> Self {
        Self {
            writer,
            // runlength: 0,
            pixels_in: 0,
            pixels_count,
            prev_pixel: Pixel::default(),

            remainder: smallvec::SmallVec::with_capacity(3),
        }
    }

    pub fn new_lz4(writer: W, pixels_count: usize) -> Self {
        let mut frame_info = lz4_flex::frame::FrameInfo::default();
        frame_info.block_checksums = true;
        frame_info.block_size = lz4_flex::frame::BlockSize::Max64KB;
        frame_info.block_mode = lz4_flex::frame::BlockMode::Linked;
        frame_info.content_checksum = true;
        frame_info.content_size = Some((pixels_count * C) as u64);

        Self::new(
            Writer::Lz4Encoder(FrameEncoder::with_frame_info(frame_info, writer)),
            pixels_count,
        )
    }

    pub fn new_uncompressed(writer: W, pixels_count: usize) -> Self {
        Self::new(Writer::UncompressedEncoder(writer), pixels_count)
    }

    #[inline]
    fn encode_pixel(&mut self, curr_pixel: Pixel<C>, prev_pixel: Pixel<C>) -> std::io::Result<()> {
        self.pixels_in += 1;
        // alpha diff encoding (whenever only alpha channel changes)
        if (C == 2 || C == 4) && curr_pixel.rgb() == prev_pixel.rgb() {
            if let Some(diff) = prev_pixel.alpha_diff(&curr_pixel) {
                self.writer.write_one(diff)?;
                return Ok(());
            }
        }

        let is_gray = curr_pixel.is_gray();
        if C != 1 && curr_pixel.a() != prev_pixel.a() && curr_pixel.a() != 255 {
            if is_gray {
                // Gray Alpha encoding (whenever alpha channel changes and pixel is gray)
                self.writer
                    .write_all(&[Op::GrayAlpha as u8, curr_pixel.r(), curr_pixel.a()])?;
            } else {
                if C != Channels::Rgba as u8 as usize {
                    panic!("RGBA encoding is only supported for RGBA images");
                }

                // RGBA encoding (whenever alpha channel changes)
                self.writer.write_all(&[
                    Op::Rgba as u8,
                    curr_pixel.r(),
                    curr_pixel.g(),
                    curr_pixel.b(),
                    curr_pixel.a(),
                ])?;
            }

            return Ok(());
        }

        // Difference between current and previous pixel
        let diff = curr_pixel.diff(&prev_pixel);

        // Diff encoding
        if let Some(diff) = diff.color() {
            self.writer.write_one(diff)?;
            return Ok(());
        }

        if is_gray {
            // Gray encoding
            self.writer.write_one(Op::Gray as u8)?;
            self.writer.write_one(curr_pixel.r())?;
            return Ok(());
        }

        // Luma encoding (a little bit broken on fast_decode) TODO: fix
        if let Some(luma) = diff.luma() {
            self.writer.write_all(&luma)?;
            return Ok(());
        }

        // RGB encoding
        self.writer.write_all(&[
            Op::Rgb as u8,
            curr_pixel.r(),
            curr_pixel.g(),
            curr_pixel.b(),
        ])?;
        Ok(())
    }

    // flushes the remaining pixels and writes the end of image marker, automatically called after N pixels are encoded
    pub fn finish(&mut self) -> std::io::Result<()> {
        self.writer.write_all(&END_OF_IMAGE)
    }

    // take a reader and encode it pixel by pixel
    pub fn encode<R: Read>(&mut self, mut reader: R) -> std::io::Result<u64> {
        io::copy(&mut reader, self)
    }

    #[inline]
    fn write_aligned(&mut self, buf: &[u8]) -> std::io::Result<()> {
        for chunk in buf.chunks_exact(C) {
            let curr_pixel: Pixel<C> = chunk.into();

            self.encode_pixel(curr_pixel, self.prev_pixel)?;
            self.prev_pixel = curr_pixel;
        }

        if buf.len() % C != 0 {
            // save the remainder for the next write
            self.remainder = buf[buf.len() - (buf.len() % C)..].into();
        }

        Ok(())
    }
}

impl<W: Write, const C: usize> Write for PixelEncoder<W, C> {
    // Currently always buffers C bytes before encoding a pixel, this could be improved by only buffering the remaining bytes until the next pixel boundary is reached
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // append the remainder from the previous write to the beginning of the buffer
        if !self.remainder.is_empty() {
            let mut new_buf = self.remainder.clone();
            new_buf.extend_from_slice(buf);
            self.remainder.clear();
            self.write_aligned(&new_buf)?;
        } else {
            self.write_aligned(buf)?
        };

        if self.pixels_in == self.pixels_count {
            self.finish()?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()?;

        if !self.remainder.is_empty() {
            println!("remainder buffer not empty, are the amount of channels correct?");
            println!("remainder: {:?}", self.remainder);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "buffer not empty",
            ))
        } else {
            Ok(())
        }
    }
}
