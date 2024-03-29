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

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            Reader::UncompressedDecoder(reader) => reader.read_exact(buf),
            Reader::Lz4Decoder(decoder) => decoder.read_exact(buf),
        }
    }
}

impl<R: Read> Reader<R> {
    pub fn read_bytes<const N: usize>(&mut self) -> std::io::Result<[u8; N]> {
        let mut buf = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}
