use lz4_flex::frame::FrameDecoder;
use std::io::{self, BufReader, Read, Write};

use super::reader::Reader;
use crate::{types::*, util::pixel_hash};

pub struct PixelDecoder<R: Read, const C: usize> {
    read_decoder: Reader<R>,
    cache: [RgbaColor; CACHE_SIZE],
    last_px: RgbaColor,
    pixels_in: usize,    // pixels decoded so far
    pixels_count: usize, // total number of pixels in the image
}

impl<R: Read, const C: usize> PixelDecoder<R, C> {
    pub fn new(data: Reader<R>, pixels_count: usize) -> Self {
        Self {
            read_decoder: data,
            cache: [RgbaColor([0, 0, 0, 0]); CACHE_SIZE],
            last_px: RgbaColor([0, 0, 0, 255]),
            pixels_in: 0,
            pixels_count,
        }
    }

    pub fn new_lz4(data: R, pixels_count: usize) -> Self {
        Self::new(Reader::Lz4Decoder(FrameDecoder::new(data)), pixels_count)
    }

    pub fn new_uncompressed(data: R, pixels_count: usize) -> Self {
        Self::new(Reader::UncompressedDecoder(data), pixels_count)
    }

    // take a writer and decode the image into it
    pub fn decode<W: Write>(&mut self, mut writer: W) -> std::io::Result<u64> {
        io::copy(self, &mut writer)
    }

    // take a writer and decode the image into it using a buffered reader to improve performance for io heavy tasks
    pub fn decode_buffered<W: Write>(&mut self, mut writer: W) -> std::io::Result<u64> {
        io::copy(&mut BufReader::new(self), &mut writer)
    }

    fn handle_end_of_image(&mut self) -> std::io::Result<()> {
        let mut padding = Vec::with_capacity(8);
        self.read_decoder.read_to_end(&mut padding)?;
        // last 8 bytes should be END_OF_IMAGE
        if padding[padding.len() - 8..] != END_OF_IMAGE {
            println!("padding: {:?}", padding);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid end of image",
            ));
        }

        Ok(())
    }
}

// implement read trait for Decoder
impl<R: Read, const C: usize> Read for PixelDecoder<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pixels_in >= self.pixels_count {
            println!("end of image");
            println!("pixels in: {}", self.pixels_in);
            self.handle_end_of_image()?;
            return Ok(0);
        }

        let [b1] = self.read_decoder.read::<1>()?;
        let mut pixel = RgbaColor([0, 0, 0, 255]);

        match b1 {
            OP_INDEX..=OP_INDEX_END => {
                self.last_px = self.cache[b1 as usize];
                buf[..C].copy_from_slice(&self.last_px.0[..C]);
                self.pixels_in += 1;
                return Ok(C);
            }
            OP_GRAY => {
                let b2 = self.read_decoder.read::<1>()?[0];
                pixel.0[..3].copy_from_slice(&[b2, b2, b2]);
            }
            OP_GRAY_ALPHA => {
                let b2 = self.read_decoder.read::<1>()?[0];
                let b3 = self.read_decoder.read::<1>()?[0];
                pixel.0[..4].copy_from_slice(&[b2, b2, b2, b3]);
            }
            OP_RGB => {
                pixel.0[..3].copy_from_slice(&self.read_decoder.read::<3>()?);
            }
            OP_RGBA if C >= Channels::Rgba as u8 as usize => {
                pixel.0[..4].copy_from_slice(&self.read_decoder.read::<4>()?);
            }
            OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => {
                pixel = self.last_px.apply_alpha_diff(b1);
            }
            OP_DIFF..=OP_DIFF_END => {
                pixel = self.last_px.apply_diff(b1);
            }
            OP_LUMA..=OP_LUMA_END => {
                let b2 = self.read_decoder.read::<1>()?[0];
                pixel = self.last_px.apply_luma(b1, b2);
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid opcode",
                ));
            }
        };

        buf[..C].copy_from_slice(&pixel.0[..C]);
        self.pixels_in += 1;
        self.last_px = pixel;
        self.cache[pixel_hash(pixel) as usize] = pixel;

        Ok(C)
    }
}
