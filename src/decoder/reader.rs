use std::io::Read;

use lz4_flex::frame::FrameDecoder;

pub enum Reader<R: Read> {
    UncompressedDecoder(R),
    Lz4Decoder(FrameDecoder<R>),
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Reader::UncompressedDecoder(reader) => reader.read(buf),
            Reader::Lz4Decoder(decoder) => decoder.read(buf),
        }
    }
}
