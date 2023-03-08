use lz4_flex::frame::FrameEncoder;
use std::io::{BufWriter, Write};

pub enum Writer<W: Write> {
    Lz4Encoder(Box<FrameEncoder<W>>),
    UncompressedEncoder(BufWriter<W>), // FrameEncoder already buffers internally, so for consistency we also use BufWriter here
}

pub struct Encoder<W: Write> {
    write_encoder: Writer<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            write_encoder: Writer::Lz4Encoder(Box::new(FrameEncoder::new(writer))),
        }
    }

    pub fn new_uncompressed(writer: W) -> Self {
        Self {
            write_encoder: Writer::UncompressedEncoder(BufWriter::new(writer)),
        }
    }
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.write_encoder {
            Writer::Lz4Encoder(ref mut encoder) => encoder.write(buf),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.write_encoder {
            Writer::Lz4Encoder(ref mut encoder) => encoder.flush(),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.flush(),
        }
    }
}
