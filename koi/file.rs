use std::io::{Read, Write};

use crate::{
    types::{Channels, Compression, MAGIC},
    QoirDecodeError, QoirEncodeError,
};
use bson::{Binary, Document};

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub version: u32,             // v
    pub exif: Option<Vec<u8>>,    // e
    pub width: u32,               // w
    pub height: u32,              // h
    pub channels: Channels,       // c
    pub compression: Compression, // x

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
    pub fn new(
        version: u32,
        exif: Option<Vec<u8>>,
        width: u32,
        height: u32,
        channels: Channels,
        compression: Compression,
        block_size: Option<u32>,
    ) -> FileHeader {
        FileHeader {
            version,
            exif,
            width,
            height,
            channels,
            compression,
            block_size,
        }
    }

    fn doc(&self) -> Document {
        let mut doc = Document::new();
        doc.insert("v", self.version as i32);
        doc.insert("w", self.width as i32);
        doc.insert("h", self.height as i32);
        doc.insert("c", self.channels as i32);
        doc.insert("x", self.compression as i32);

        if let Some(exif) = &self.exif {
            doc.insert("e", to_binary(exif.clone()));
        }

        doc
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), QoirEncodeError> {
        writer.write_all(&MAGIC)?;
        Ok(self.doc().to_writer(writer)?)
    }

    pub fn write_to_buf(&self, buf: &mut [u8]) -> Result<usize, QoirEncodeError> {
        buf[..MAGIC.len()].copy_from_slice(&MAGIC);

        let mut header = vec![];
        self.doc().to_writer(&mut header)?;
        buf[MAGIC.len()..MAGIC.len() + header.len()].copy_from_slice(&header);
        Ok(header.len() + MAGIC.len())
    }

    pub fn write_to_vec(&self) -> Result<Vec<u8>, QoirEncodeError> {
        let mut bytes = vec![];
        self.write(&mut bytes)?;
        Ok(bytes)
    }

    pub fn check_magic(reader: &mut dyn Read) -> Result<(), QoirDecodeError> {
        let mut magic = [0u8; MAGIC.len()];
        reader.read_exact(&mut magic).map_err(|_| {
            QoirDecodeError::InvalidFileHeader("Failed to read magic number".to_string())
        })?;

        if magic != MAGIC {
            return Err(QoirDecodeError::InvalidFileHeader(
                "Invalid magic number".to_string(),
            ));
        }

        Ok(())
    }

    pub fn parse(reader: &mut dyn Read) -> Result<FileHeader, QoirDecodeError> {
        FileHeader::check_magic(reader)?;

        let doc = Document::from_reader(reader).map_err(err("Failed to read file header"))?;

        let (version, exif, width, height, channels, compression, block_size) = (
            doc.get_i32("v")
                .map_err(err("Failed to read file version"))? as u32,
            doc.get_binary_generic("e").ok().map(|b| b.to_vec()),
            doc.get_i32("w").map_err(err("Failed to read width"))? as u32,
            doc.get_i32("h").map_err(err("Failed to read height"))? as u32,
            doc.get_i32("c").map_err(err("Failed to read channels"))? as u32,
            doc.get_i32("x")
                .map_err(err("Failed to read compression"))? as u32,
            doc.get_i32("b").ok().map(|b| b as u32),
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
        })
    }
}

fn err<F>(e: &str) -> impl FnOnce(F) -> QoirDecodeError + '_ {
    |_| QoirDecodeError::InvalidFileHeader(e.to_string())
}
