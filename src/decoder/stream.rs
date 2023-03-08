use lz4_flex::frame::FrameDecoder;
use std::io::Read;

pub enum Reader<R: Read> {
    UncompressedDecoder(R),
    Lz4Decoder(FrameDecoder<R>),
}

pub struct Decoder<R: Read> {
    read_decoder: Reader<R>,
}

impl<R: Read> Decoder<R> {
    pub fn new(data: R) -> Self {
        Self {
            read_decoder: Reader::Lz4Decoder(FrameDecoder::new(data)),
        }
    }

    pub fn new_uncompressed(data: R) -> Self {
        Self {
            read_decoder: Reader::UncompressedDecoder(data),
        }
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Reader::UncompressedDecoder(reader) => reader.read(buf),
            Reader::Lz4Decoder(decoder) => decoder.read(buf),
        }
    }
}

// implement read trait for Decoder
impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_decoder.read(buf)
    }
}
