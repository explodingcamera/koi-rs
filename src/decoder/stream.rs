use lz4_flex::frame::FrameDecoder;
use std::io::Read;

use super::reader::Reader;
use crate::types::RgbaColor;

pub struct PixelDecoder<R: Read> {
    read_decoder: Reader<R>,
    op_data: u8,
    op_pos: u8,
    pixels_in: usize,
    pixels: usize,
    px: RgbaColor,
    px_prev: RgbaColor,
    index: [RgbaColor; 64],
}

impl<R: Read> PixelDecoder<R> {
    pub fn new(data: Reader<R>) -> Self {
        Self {
            read_decoder: data,
            index: [RgbaColor([0, 0, 0, 0]); 64],
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
impl<R: Read> Read for PixelDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_decoder.read(buf)
    }
}
