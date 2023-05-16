use std::io::{Read, Write};

use crate::{
    types::{Channels, Compression, MAGIC},
    util::{Buffer, BufferMut, Writer},
    KoiDecodeError, KoiEncodeError,
};
use bson::{Binary, Document};

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub version: u32,             // v
    pub exif: Option<Vec<u8>>,    // e
    pub width: u64,               // w
    pub height: u64,              // h
    pub channels: Channels,       // c
    pub compression: Compression, // x
    pub color_space: u32,         // s

    // defaults to a dynamic value based on the image size if not specified (version >= 1)
    pub block_size: Option<u32>, // b
}

#[inline]
fn to_binary(bytes: Vec<u8>) -> Binary {
    Binary {
        bytes,
        subtype: bson::spec::BinarySubtype::Generic,
    }
}

impl FileHeader {
    pub fn min_output_size(&self) -> usize {
        self.width as usize * self.height as usize * self.channels as usize * self.channels as usize
    }

    pub fn new(
        version: u32,
        exif: Option<Vec<u8>>,
        width: u64,
        height: u64,
        channels: Channels,
        compression: Compression,
        block_size: Option<u32>,
        color_space: Option<u32>,
    ) -> FileHeader {
        FileHeader {
            version,
            exif,
            width,
            height,
            channels,
            compression,
            block_size,
            color_space: color_space.unwrap_or(0),
        }
    }

    fn doc(&self) -> Document {
        let mut doc = Document::new();
        doc.insert("v", self.version as i32);
        doc.insert("w", self.width as i64);
        doc.insert("h", self.height as i64);
        doc.insert("c", self.channels as i32);
        doc.insert("x", self.compression as i32);
        doc.insert("s", self.color_space as i32);

        if let Some(exif) = &self.exif {
            doc.insert("e", to_binary(exif.clone()));
        }

        doc
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), KoiEncodeError> {
        writer.write_all(&MAGIC)?;
        Ok(self.doc().to_writer(writer)?)
    }

    pub fn write_to_buf<'a>(&self, buf: BufferMut<'a>) -> Result<BufferMut<'a>, KoiEncodeError> {
        Ok(buf.write_many(&self.write_to_vec()?))
    }

    pub fn write_to_vec(&self) -> Result<Vec<u8>, KoiEncodeError> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;
        Ok(bytes)
    }

    pub fn check_magic(reader: &mut dyn Read) -> Result<(), KoiDecodeError> {
        let mut magic = [0u8; MAGIC.len()];
        reader.read_exact(&mut magic).map_err(|_| {
            KoiDecodeError::InvalidFileHeader("Failed to read magic number".to_string())
        })?;

        if magic != MAGIC {
            return Err(KoiDecodeError::InvalidFileHeader(
                "Invalid magic number".to_string(),
            ));
        }

        Ok(())
    }

    #[allow(clippy::all)] // clippy is making the code slower
    pub fn read_buf<'a>(buf: Buffer<'a>) -> Result<(Buffer<'a>, FileHeader), KoiDecodeError> {
        let (len, header) = FileHeader::read_bytes(&buf)?;
        Ok((buf.advance(len), header))
    }

    pub fn read_bytes(bytes: &[u8]) -> Result<(usize, FileHeader), KoiDecodeError> {
        let mut reader = std::io::Cursor::new(bytes);
        FileHeader::read(&mut reader).map(|header| (reader.position() as usize, header))
    }

    pub fn read(reader: &mut dyn Read) -> Result<FileHeader, KoiDecodeError> {
        FileHeader::check_magic(reader)?;

        let doc = Document::from_reader(reader).map_err(err("Failed to read file header"))?;

        let (version, exif, width, height, channels, compression, block_size, color_space) = (
            doc.get_i32("v")
                .map_err(err("Failed to read file version"))? as u32,
            doc.get_binary_generic("e").ok().map(|b| b.to_vec()),
            doc.get_i64("w").map_err(err("Failed to read width"))? as u64,
            doc.get_i64("h").map_err(err("Failed to read height"))? as u64,
            doc.get_i32("c").map_err(err("Failed to read channels"))? as u32,
            doc.get_i32("x")
                .map_err(err("Failed to read compression"))? as u32,
            doc.get_i32("b").ok().map(|b| b as u32),
            doc.get_i32("s").ok().map(|b| b as u32),
        );

        Ok(Self {
            version,
            exif,
            width,
            height,
            channels: u8::try_from(channels)
                .map_err(err("Invalid channels"))?
                .try_into()
                .map_err(err("Invalid channels"))?,
            compression: u8::try_from(compression)
                .map_err(err("Invalid compression"))?
                .try_into()
                .map_err(err("Invalid compression"))?,
            block_size,
            color_space: color_space.unwrap_or(0),
        })
    }
}

fn err<F>(e: &str) -> impl FnOnce(F) -> KoiDecodeError + '_ {
    |_| KoiDecodeError::InvalidFileHeader(e.to_string())
}
