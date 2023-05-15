use lz4_flex::frame::FrameDecoder;
use std::io::{self, BufReader, Read, Write};

use super::reader::Reader;
use crate::{
    types::*,
    util::{cold, likely, unlikely},
};

pub struct PixelDecoder<R: Read, const C: usize> {
    read_decoder: Reader<R>,
    last_px: Pixel<C>,
    pixels_in: usize,    // pixels decoded so far
    pixels_count: usize, // total number of pixels in the image
}

impl<R: Read, const C: usize> PixelDecoder<R, C> {
    pub fn new(data: Reader<R>, pixels_count: usize) -> Self {
        Self {
            read_decoder: data,
            last_px: Pixel::default(),
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

        // last 4 bytes should be END_OF_IMAGE
        if padding[..] != END_OF_IMAGE[..] {
            println!("padding: {:?}", padding);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid end of image",
            ));
        }

        Ok(())
    }

    // #[inline]
    // // somehow the faster version sometimes has issues with the hash function
    // fn decode_pixels(&mut self, buf: &mut [u8], count: usize) -> std::io::Result<usize> {
    //     let mut pixels_read = 0;

    //     if self.pixels_in >= self.pixels_count {
    //         self.handle_end_of_image()?;
    //         return Ok(0);
    //     }

    //     let count = std::cmp::min(count, self.pixels_count - self.pixels_in);
    //     for i in 0..count {
    //         let [b1] = self.read_decoder.read_bytes::<1>()?;
    //         let buffer_offset = i * C;

    //         let pixel: Pixel<C> = match b1 {
    //             OP_INDEX..=OP_INDEX_END => {
    //                 self.last_px = self.cache[b1 as usize];
    //                 buf[buffer_offset..buffer_offset + C].copy_from_slice(&self.last_px.data);
    //                 self.pixels_in += 1;
    //                 pixels_read += 1;
    //                 continue;
    //             }
    //             OP_GRAY => {
    //                 let [b2] = self.read_decoder.read_bytes::<1>()?;
    //                 Pixel::from_grayscale(b2)
    //             }
    //             OP_GRAY_ALPHA => {
    //                 if unlikely(C < Channels::GrayAlpha as u8 as usize) {
    //                     return Err(std::io::Error::new(
    //                         std::io::ErrorKind::InvalidData,
    //                         format!("Invalid opcode GRAY_ALPHA for {} channels", C),
    //                     ));
    //                 }

    //                 let [b2, b3] = self.read_decoder.read_bytes::<2>()?;
    //                 Pixel::from([b2, b2, b2, b3])
    //             }
    //             OP_RGB => {
    //                 let [b2, b3, b4] = self.read_decoder.read_bytes::<3>()?;
    //                 Pixel::from([b2, b3, b4])
    //             }
    //             OP_RGBA => {
    //                 if unlikely(C < Channels::Rgba as u8 as usize) {
    //                     return Err(std::io::Error::new(
    //                         std::io::ErrorKind::InvalidData,
    //                         format!("Invalid opcode RGBA for {} channels", C),
    //                     ));
    //                 }
    //                 Pixel::from(self.read_decoder.read_bytes::<4>()?)
    //             }
    //             OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => self.last_px.apply_alpha_diff(b1),
    //             OP_DIFF..=OP_DIFF_END => self.last_px.apply_diff(b1),
    //             OP_LUMA..=OP_LUMA_END => {
    //                 let [b2] = self.read_decoder.read_bytes::<1>()?;
    //                 self.last_px.apply_luma(b1, b2)
    //             }
    //         };

    //         buf[buffer_offset..buffer_offset + C].copy_from_slice(&pixel.data);
    //         self.pixels_in += 1;
    //         self.last_px = pixel;
    //         self.cache[pixel.hash() as usize] = pixel;
    //         pixels_read += 1;
    //     }

    //     Ok(pixels_read * C)
    // }

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
            self.handle_end_of_image()?;
        }

        while likely(buffer_pos < buffer_len && self.pixels_in < self.pixels_count && !buffer_empty)
        {
            let buffer_offset = pixels_read * C;
            let required_bytes = Self::get_required_bytes(buffer[buffer_pos]);
            let available_bytes = buffer_len - 1 - buffer_pos;

            if unlikely(required_bytes > available_bytes) {
                buffer_empty = true;
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

            buffer_pos =
                self.read_pixel(&buffer, buffer_pos, buf, buffer_offset, required_bytes)?;
            pixels_read += 1;
        }

        Ok(pixels_read * C)
    }

    fn get_required_bytes(opcode: u8) -> usize {
        match opcode {
            // OP_INDEX..=OP_INDEX_END => 0,
            OP_GRAY => 1,
            OP_GRAY_ALPHA => 2,
            OP_RGB => 3,
            OP_RGBA => 4,
            OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => 0,
            OP_DIFF..=OP_DIFF_END => 0,
            OP_LUMA..=OP_LUMA_END => 1,
            _ => {
                cold();
                panic!("Invalid opcode {}", opcode)
            }
        }
    }

    // returns the number of bytes read from the input buffer, always writes C bytes to the output buffer
    #[inline]
    fn read_pixel(
        &mut self,
        buf_in: &[u8],
        buffer_in_pos: usize,
        buf_out: &mut [u8],
        buffer_offset: usize,
        required_bytes: usize,
    ) -> io::Result<usize> {
        let b1 = buf_in[buffer_in_pos];

        let pixel: Pixel<C> = match b1 {
            // OP_INDEX..=OP_INDEX_END => {
            //     self.last_px = self.cache[b1 as usize];
            //     buf_out[buffer_offset..buffer_offset + C].copy_from_slice(&self.last_px.data);
            //     self.pixels_in += 1;
            //     return Ok(buffer_in_pos + 1);
            // }
            OP_GRAY => {
                let b2 = buf_in[buffer_in_pos + 1];
                Pixel::from_grayscale(b2)
            }
            OP_GRAY_ALPHA => {
                let b2 = buf_in[buffer_in_pos + 1];
                let b3 = buf_in[buffer_in_pos + 2];
                Pixel::from([b2, b2, b2, b3])
            }
            OP_RGB => {
                let r = buf_in[buffer_in_pos + 1];
                let g = buf_in[buffer_in_pos + 2];
                let b = buf_in[buffer_in_pos + 3];
                Pixel::from([r, g, b])
            }
            OP_RGBA => {
                if unlikely(C < Channels::Rgba as u8 as usize) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid opcode RGBA for {} channels", C),
                    ));
                }
                buf_in[buffer_in_pos + 1..buffer_in_pos + 5].into()
            }
            OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END => self.last_px.apply_alpha_diff(b1),
            OP_DIFF..=OP_DIFF_END => self.last_px.apply_diff(b1),
            OP_LUMA..=OP_LUMA_END => {
                let b2 = buf_in[buffer_in_pos + 1];
                self.last_px.apply_luma(b1, b2)
            }
            _ => {
                cold();
                panic!("Invalid opcode {}", b1)
            }
        };

        buf_out[buffer_offset..buffer_offset + C].copy_from_slice(&pixel.data);
        self.pixels_in += 1;
        self.last_px = pixel;

        Ok(buffer_in_pos + required_bytes + 1)
    }

    pub fn read_all_pixels_buf(&mut self, input: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let mut input_decoded = Vec::with_capacity(input.len());
        self.read_decoder.read_to_end(&mut input_decoded)?;

        let mut buffer_pos = 0;
        while self.pixels_in < self.pixels_count {
            let required_bytes = Self::get_required_bytes(input_decoded[buffer_pos]);

            buffer_pos = self.read_pixel(
                &input_decoded,
                buffer_pos,
                output,
                self.pixels_in * C,
                required_bytes,
            )?;
        }

        self.handle_end_of_image()?;
        Ok(self.pixels_in * C + 8)
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
