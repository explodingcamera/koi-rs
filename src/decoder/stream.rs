use lz4_flex::frame::FrameDecoder;
use std::io::Read;

use super::reader::Reader;
use crate::types::*;

pub struct PixelDecoder<R: Read, const C: usize> {
    read_decoder: Reader<R>,
    op_data: u8,
    op_pos: u8,
    pixels_in: usize,
    pixels: usize,
    px: RgbaColor,
    px_prev: RgbaColor,
    cache: [RgbaColor; 64],
}

impl<R: Read, const C: usize> PixelDecoder<R, C> {
    pub fn new(data: Reader<R>) -> Self {
        Self {
            read_decoder: data,
            cache: [RgbaColor([0, 0, 0, 0]); 64],
            op_data: 0,
            op_pos: 0,
            pixels_in: 0,
            pixels: 0,
            px: RgbaColor([0, 0, 0, 0]),
            px_prev: RgbaColor([0, 0, 0, 0]),
        }
    }

    pub fn new_lz4(data: R) -> Self {
        Self::new(Reader::Lz4Decoder(FrameDecoder::new(data)))
    }

    pub fn new_uncompressed(data: R) -> Self {
        Self::new(Reader::UncompressedDecoder(data))
    }
}

// implement read trait for Decoder
impl<R: Read, const C: usize> Read for PixelDecoder<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let b1 = self.read_decoder.read_u8()?;
        match b1 {
            OP_INDEX..=OP_INDEX_END => {}
            OP_RGB => {}
            OP_RGBA if C >= Channels::Rgba as u8 as usize => {}
            OP_RUNLENGTH..=OP_RUNLENGTH_END => {}
            OP_DIFF..=OP_DIFF_END => {}
            OP_LUMA..=OP_LUMA_END => {}
            _ => {}
        }

        Ok(0)
    }
}
