use super::writer::Writer;
use crate::types::RgbaColor;
use lz4_flex::frame::FrameEncoder;
use std::io::{BufWriter, Write};

pub struct PixelEncoder<W: Write> {
    write_encoder: Writer<W>,

    pos: u8,
    len: u8,
    pixels_in: usize,
    pixels: usize,
    px: RgbaColor,
    px_prev: RgbaColor,
    index: [RgbaColor; 64],
}

impl<W: Write> PixelEncoder<W> {
    pub fn new(writer: Writer<W>) -> Self {
        Self {
            write_encoder: writer,
            index: [RgbaColor(0, 0, 0, 0); 64],
            pos: 0,
            len: 0,
            pixels_in: 0,
            pixels: 0,
            px: RgbaColor(0, 0, 0, 0),
            px_prev: RgbaColor(0, 0, 0, 0),
        }
    }

    pub fn new_lz4(writer: W) -> Self {
        Self::new(Writer::Lz4Encoder(Box::new(FrameEncoder::new(writer))))
    }

    pub fn new_uncompressed(writer: W) -> Self {
        Self::new(Writer::UncompressedEncoder(BufWriter::new(writer)))
    }
}

impl<W: Write> Write for PixelEncoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.write_encoder.flush()
    }
}
