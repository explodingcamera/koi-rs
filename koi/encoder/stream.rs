use super::writer::Writer;
use crate::{
    types::{color_diff, luma_diff, Channels, Op, RgbaColor, CACHE_SIZE, END_OF_IMAGE},
    util::{pixel_hash, unlikely},
};
use lz4_flex::frame::FrameEncoder;
use std::io::{self, Read, Write};

// pub struct PixelEncoderConfig {
//     pub index_encoding: bool,
//     pub diff_encoding: bool,
//     pub luma_encoding: bool,
//     pub alpha_diff_encoding: bool,
// }

// PixelEncoder is a stream encoder that encodes pixels one by one
// - Writer is a wrapper around the underlying writer that can be either a lz4 encoder or a regular writer
// - C is the number of channels in the image
pub struct PixelEncoder<W: Write, const C: usize> {
    writer: Writer<W>,
    // runlength: u8,    // if runlength > 0 then we are in runlength encoding mode
    pixels_in: usize, // pixels encoded so far
    pixels_count: usize,

    cache: [RgbaColor; CACHE_SIZE],
    prev_pixel: RgbaColor,

    remainder: smallvec::SmallVec<[u8; 3]>,
}

impl<W: Write, const C: usize> PixelEncoder<W, C> {
    pub fn new(writer: Writer<W>, pixels_count: usize) -> Self {
        Self {
            writer,
            cache: [RgbaColor([0, 0, 0, 0]); CACHE_SIZE],
            // runlength: 0,
            pixels_in: 0,
            pixels_count,
            prev_pixel: RgbaColor([0, 0, 0, 0]),

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
            Writer::Lz4Encoder(Box::new(FrameEncoder::with_frame_info(frame_info, writer))),
            pixels_count,
        )
    }

    pub fn new_uncompressed(writer: W, pixels_count: usize) -> Self {
        Self::new(Writer::UncompressedEncoder(writer), pixels_count)
    }

    #[inline]
    fn encode_pixel(
        &mut self,
        curr_pixel: RgbaColor,
        prev_pixel: RgbaColor,
    ) -> std::io::Result<()> {
        if unlikely(C < Channels::Rgba as u8 as usize) {
            // alpha channel should be 255 for all pixels in RGB images
            assert_eq!(
                curr_pixel.0[3], 255,
                "alpha channel should be 255 for all pixels in RGB images"
            );
        };

        self.pixels_in += 1;
        let mut curr_pixel = curr_pixel;

        // index encoding
        let hash = pixel_hash(curr_pixel);
        if self.cache[hash as usize] == curr_pixel {
            self.cache_pixel(&mut curr_pixel, hash);
            self.writer.write_one(u8::from(Op::Index) | hash)?;
            return Ok(());
        }

        // alpha diff encoding (whenever only alpha channel changes)
        if curr_pixel.0[..3] == prev_pixel.0[..3] && curr_pixel.0[3] != prev_pixel.0[3] {
            if let Some(diff) = prev_pixel.alpha_diff(&curr_pixel) {
                self.cache_pixel(&mut curr_pixel, hash);
                self.writer.write_one(diff)?;
                return Ok(());
            }
        }

        let is_gray = curr_pixel.is_gray();
        if curr_pixel.0[3] != prev_pixel.0[3] && curr_pixel.0[3] != 255 {
            if is_gray {
                // Gray Alpha encoding (whenever alpha channel changes and pixel is gray)
                self.cache_pixel(&mut curr_pixel, hash);
                self.writer.write_one(Op::GrayAlpha as u8)?;
                self.writer.write_all(&[curr_pixel.0[0], curr_pixel.0[3]])?;
                return Ok(());
            } else {
                if C != Channels::Rgba as u8 as usize {
                    panic!("RGBA encoding is only supported for RGBA images");
                }
                // RGBA encoding (whenever alpha channel changes)
                self.cache_pixel(&mut curr_pixel, hash);
                self.writer.write_one(Op::Rgba as u8)?;
                self.writer.write_all(&curr_pixel.0)?;
                return Ok(());
            }
        }

        // // Difference between current and previous pixel
        let diff = curr_pixel.diff(&prev_pixel);

        // Diff encoding
        if let Some(diff) = color_diff(diff) {
            self.cache_pixel(&mut curr_pixel, hash);
            self.writer.write_one(diff)?;
            return Ok(());
        }

        // Luma encoding (broken on fast_decode)
        // if let Some(luma) = luma_diff(diff) {
        //     self.cache_pixel(&mut curr_pixel, hash);
        //     self.writer.write_all(&luma)?;
        //     return Ok(());
        // }

        if is_gray {
            println!("is_gray {}", curr_pixel.0[0]);
            // Gray encoding
            let RgbaColor([r, g, b, _]) = curr_pixel;
            if r == g && g == b {
                self.cache_pixel(&mut curr_pixel, hash);
                self.writer.write_one(Op::Gray as u8)?;
                self.writer.write_one(curr_pixel.0[0])?;
                return Ok(());
            }
        }

        // RGB encoding
        let RgbaColor([r, g, b, _]) = curr_pixel;
        self.cache_pixel(&mut curr_pixel, hash);
        self.writer.write_all(&[Op::Rgb as u8, r, g, b])?;
        Ok(())
    }

    #[inline]
    fn cache_pixel(&mut self, curr_pixel: &mut RgbaColor, hash: u8) {
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

    #[inline]
    fn write_aligned(&mut self, buf: &[u8]) -> std::io::Result<()> {
        for chunk in buf.chunks_exact(C) {
            let mut curr_pixel = RgbaColor([0, 0, 0, 255]);
            curr_pixel.0[..C].copy_from_slice(chunk);
            self.encode_pixel(curr_pixel, self.prev_pixel)?;
            self.prev_pixel = curr_pixel;
            // we've reached the end of the buffer and don't have enough bytes to fill a pixel
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
        // println!("buf.len(): {}", buf.len());

        // append the remainder from the previous write to the beginning of the buffer
        if !self.remainder.is_empty() {
            let mut new_buf = self.remainder.clone();
            new_buf.extend_from_slice(buf);
            self.remainder.clear();
            self.write_aligned(&new_buf)?;
        } else {
            self.write_aligned(buf)?
        };

        // println!(
        //     "pixels_in: {}, pixels_count: {}",
        //     self.pixels_in, self.pixels_count
        // );

        if self.pixels_in == self.pixels_count {
            // println!("writing end of image marker");
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
