use lz4_flex::frame::FrameDecoder;
use std::io::{self, BufReader, Read, Write};

use super::reader::Reader;
use crate::{
    types::*,
    util::{likely, pixel_hash, unlikely},
};

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

    #[allow(dead_code)]
    fn handle_end_of_image(&mut self) -> std::io::Result<()> {
        let mut padding = [0; 8];
        self.read_decoder.read_exact(&mut padding)?;

        // last 8 bytes should be END_OF_IMAGE
        if padding[0..] != END_OF_IMAGE {
            println!("padding: {:?}", padding);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid end of image",
            ));
        }

        Ok(())
    }

    #[allow(dead_code)]
    #[inline]
    fn decode_pixels(&mut self, buf: &mut [u8], count: usize) -> std::io::Result<usize> {
        let mut pixels_read = 0;

        if self.pixels_in >= self.pixels_count {
            self.handle_end_of_image()?;
            return Ok(0);
        }

        let count = std::cmp::min(count, self.pixels_count - self.pixels_in);
        for i in 0..count {
            let [b1] = self.read_decoder.read_bytes::<1>()?;
            let buffer_offset = i * C;

            let pixel = match b1 {
                OP_INDEX..=OP_INDEX_END => {
                    self.last_px = self.cache[b1 as usize];
                    buf[buffer_offset..buffer_offset + C].copy_from_slice(&self.last_px.0[..C]);
                    self.pixels_in += 1;
                    pixels_read += 1;
                    continue;
                }
                OP_GRAY => {
                    let [b2] = self.read_decoder.read_bytes::<1>()?;
                    RgbaColor::from_grayscale(b2)
                }
                OP_GRAY_ALPHA => {
                    let [b2, b3] = self.read_decoder.read_bytes::<2>()?;
                    RgbaColor([b2, b2, b2, b3])
                }
                OP_RGB => {
                    let [b2, b3, b4] = self.read_decoder.read_bytes::<3>()?;
                    RgbaColor::from_rgb([b2, b3, b4])
                }
                OP_RGBA => {
                    if unlikely(C < Channels::Rgba as u8 as usize) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid opcode RGBA for {} channels", C),
                        ));
                    }
                    RgbaColor(self.read_decoder.read_bytes::<4>()?)
                }
                OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => self.last_px.apply_alpha_diff(b1),
                OP_DIFF..=OP_DIFF_END => self.last_px.apply_diff(b1),
                OP_LUMA..=OP_LUMA_END => {
                    let [b2] = self.read_decoder.read_bytes::<1>()?;
                    self.last_px.apply_luma(b1, b2)
                }
            };

            buf[buffer_offset..buffer_offset + C].copy_from_slice(&pixel.0[..C]);
            self.pixels_in += 1;
            self.last_px = pixel;
            self.cache[pixel_hash(pixel) as usize] = pixel;
            pixels_read += 1;
        }

        Ok(pixels_read * C)
    }

    #[inline]
    fn read_pixels_fast(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut pixels_read = 0;

        let mut buffer = vec![0u8; buf.len() / 4]; // max possible compression ratio is 4:1 (without factoring in lz4 compression on top of that)
        let mut buffer_pos = 0;
        let mut buffer_len = self.read_decoder.read(&mut buffer)?;

        if buffer_len == 0 || self.pixels_in >= self.pixels_count {
            return Ok(0);
        }

        let mut buffer_empty = false;

        if self.pixels_in >= self.pixels_count {
            // get buffer_pos to end of buffer
            let mut bytes = Vec::from(&buffer[buffer_pos..buffer_len]);
            self.read_decoder.read_to_end(&mut bytes)?;

            if bytes.len() < END_OF_IMAGE.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid end of image",
                ));
            }

            // last 5 bytes should be the same as END_OF_IMAGE's last 5 bytes
            if bytes[bytes.len() - 5..] != END_OF_IMAGE[END_OF_IMAGE.len() - 5..] {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid end of image",
                ));
            }
        }

        while likely(buffer_pos < buffer_len && self.pixels_in < self.pixels_count && !buffer_empty)
        {
            let b1 = buffer[buffer_pos];

            let buffer_offset = pixels_read * C;
            let required_bytes = match b1 {
                OP_INDEX..=OP_INDEX_END => 0,
                OP_GRAY => 1,
                OP_GRAY_ALPHA => 2,
                OP_RGB => 3,
                OP_RGBA => 4,
                OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => 0,
                OP_DIFF..=OP_DIFF_END => 0,
                OP_LUMA..=OP_LUMA_END => 1,
            };

            let available_bytes = buffer_len - 1 - buffer_pos;
            let required_bytes = required_bytes as usize;

            if unlikely(required_bytes > available_bytes) {
                buffer_empty = true;
                // copy current byte and remaining bytes to the start of the buffer
                // buffer.copy_within(buffer_pos..buffer_len, 0);

                // read "required_bytes" bytes from read_decoder
                let mut new_bytes = [0u8; 4];
                let read_count = required_bytes - available_bytes;
                self.read_decoder.read_exact(&mut new_bytes[..read_count])?;

                // set buffer to current_bytes + new_bytes
                buffer = buffer[buffer_pos..].to_vec();
                buffer.extend_from_slice(&new_bytes[..read_count]);

                buffer_len = buffer.len();
                buffer_pos = 0;
            }

            let pixel = match b1 {
                OP_INDEX..=OP_INDEX_END => {
                    self.last_px = self.cache[b1 as usize];
                    buf[buffer_offset..buffer_offset + C].copy_from_slice(&self.last_px.0[..C]);
                    self.pixels_in += 1;
                    pixels_read += 1;
                    buffer_pos += 1;
                    continue;
                }
                OP_GRAY => {
                    let b2 = buffer[buffer_pos + 1];
                    RgbaColor::from_grayscale(b2)
                }
                OP_GRAY_ALPHA => {
                    let b2 = buffer[buffer_pos + 1];
                    let b3 = buffer[buffer_pos + 2];
                    RgbaColor([b2, b2, b2, b3])
                }
                OP_RGB => {
                    let r = buffer[buffer_pos + 1];
                    let g = buffer[buffer_pos + 2];
                    let b = buffer[buffer_pos + 3];
                    RgbaColor([r, g, b, 255])
                }
                OP_RGBA => {
                    if unlikely(C < Channels::Rgba as u8 as usize) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid opcode RGBA for {} channels", C),
                        ));
                    }
                    RgbaColor(buffer[buffer_pos + 1..buffer_pos + 5].try_into().unwrap())
                }
                OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => self.last_px.apply_alpha_diff(b1),
                OP_DIFF..=OP_DIFF_END => self.last_px.apply_diff(b1),
                OP_LUMA..=OP_LUMA_END => {
                    let b2 = buffer[buffer_pos + 1];
                    self.last_px.apply_luma(b1, b2)
                }
            };

            buf[buffer_offset..buffer_offset + C].copy_from_slice(&pixel.0[..C]);
            self.pixels_in += 1;
            self.last_px = pixel;
            self.cache[pixel_hash(pixel) as usize] = pixel;
            pixels_read += 1;
            buffer_pos += required_bytes + 1;
        }

        Ok(pixels_read * C)
    }
}

// implement read trait for Decoder
impl<R: Read, const C: usize> Read for PixelDecoder<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_pixels_fast(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.read_pixels_fast(buf)
    }
}
