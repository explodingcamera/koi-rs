use std::io::{BufWriter, Write};

use lz4_flex::frame::FrameEncoder;

pub enum Writer<W: Write> {
    Lz4Encoder(Box<FrameEncoder<W>>),
    UncompressedEncoder(BufWriter<W>), // FrameEncoder already buffers internally, so for consistency we also use BufWriter here
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writer::Lz4Encoder(ref mut encoder) => encoder.write(buf),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writer::Lz4Encoder(ref mut encoder) => encoder.flush(),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.flush(),
        }
    }
}
