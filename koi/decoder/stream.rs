use lz4_flex::frame::FrameDecoder;
use std::io::{self, Read, Write};

use super::reader::Reader;
use crate::{types::*, util::pixel_hash};

pub struct PixelDecoder<R: Read, const C: usize> {
    read_decoder: Reader<R>,
    cache: [RgbaColor; 64],
    last_px: RgbaColor,
    pixels_in: usize,    // pixels decoded so far
    pixels_count: usize, // total number of pixels in the image
}

impl<R: Read, const C: usize> PixelDecoder<R, C> {
    pub fn new(data: Reader<R>, pixels_count: usize) -> Self {
        Self {
            read_decoder: data,
            cache: [RgbaColor([0, 0, 0, 0]); 64],
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
}

// implement read trait for Decoder
impl<R: Read, const C: usize> Read for PixelDecoder<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut n = 1;
        let [b1] = self.read_decoder.read::<1>()?;
        let mut pixel = RgbaColor([0, 0, 0, 255]);

        if self.pixels_in >= self.pixels_count {
            println!(
                "pixels_in: {}, pixels_count: {}",
                self.pixels_in, self.pixels_count
            );
            let padding = self.read_decoder.read::<8>()?;
            if padding != END_OF_IMAGE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid end of image",
                ));
            }

            return Ok(0);
        }

        match b1 {
            OP_INDEX..=OP_INDEX_END => {
                println!("OP_INDEX");
                buf[..C].copy_from_slice(&self.cache[b1 as usize].0[..C]);
                self.last_px = self.cache[b1 as usize];
                self.pixels_in += n;
                return Ok(n);
            }
            OP_RGB => {
                println!("OP_RGB");
                pixel.0[..3].copy_from_slice(&self.read_decoder.read::<3>()?);
                n += 3;
            }
            OP_RGBA if C >= Channels::Rgba as u8 as usize => {
                println!("OP_RGBA");
                pixel.0[..4].copy_from_slice(&self.read_decoder.read::<4>()?);
                n += 4;
            }
            OP_RUNLENGTH..=OP_RUNLENGTH_END => {
                let run = ((b1 & 0x3f) as usize).min(self.pixels_count - self.pixels_in);
                println!("OP_RUNLENGTH: {}", run);
                pixel = self.last_px;
                for i in 0..run {
                    buf[i * 4..(i + 1) * 4].copy_from_slice(&pixel.0);
                }
                n += run;
            }
            OP_DIFF..=OP_DIFF_END => {
                println!("OP_DIFF");
                pixel = self.last_px.apply_diff(b1);
            }
            OP_LUMA..=OP_LUMA_END => {
                println!("OP_LUMA");
                let b2 = self.read_decoder.read::<1>()?[0];
                pixel = self.last_px.apply_luma(b1, b2);
                n += 1;
            }
            _ => {}
        };

        buf[..4].copy_from_slice(&pixel.0);
        self.cache[pixel_hash(pixel) as usize] = pixel;
        self.last_px = pixel;
        self.pixels_in += n;

        Ok(n)
    }
}
