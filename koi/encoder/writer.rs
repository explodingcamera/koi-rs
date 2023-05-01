use lz4_flex::frame::FrameEncoder;
use std::io::Write;

#[allow(clippy::large_enum_variant)]
pub enum Writer<W: Write> {
    Lz4Encoder(FrameEncoder<W>),
    UncompressedEncoder(W), // FrameEncoder already buffers internally, so for consistency we also use BufWriter here
}

impl<W: Write> Writer<W> {
    pub fn write_one(&mut self, byte: u8) -> std::io::Result<()> {
        self.write_all(&[byte])
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writer::Lz4Encoder(ref mut encoder) => encoder.write(buf),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.write(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Writer::Lz4Encoder(ref mut encoder) => encoder.write_all(buf),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.write_all(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writer::Lz4Encoder(ref mut encoder) => encoder.flush(),
            Writer::UncompressedEncoder(ref mut encoder) => encoder.flush(),
        }
    }
}
