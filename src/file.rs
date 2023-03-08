use std::io::{Read, Write};

use crate::{
    types::{Channels, Compression, MAGIC},
    QoirDecodeError, QoirEncodeError,
};
use bson::{Binary, Document};

#[derive(Debug)]
pub struct FileHeader {
    pub version: u32,
    pub data_size: u64,
    pub exif: Option<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub channels: Channels,
    pub compression: Compression,
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
        data_size: u64,
        exif: Option<Vec<u8>>,
        width: u32,
        height: u32,
        channels: Channels,
        compression: Compression,
    ) -> FileHeader {
        FileHeader {
            version: 0,
            data_size,
            exif,
            width,
            height,
            channels,
            compression,
        }
    }

    pub fn write(&self, writer: &mut dyn Write) -> Result<(), QoirEncodeError> {
        writer.write_all(MAGIC)?;

        let mut doc = Document::new();
        doc.insert("version", self.version as i32);
        doc.insert("data_size", self.data_size as i64);

        if let Some(exif) = &self.exif {
            doc.insert("exif", to_binary(exif.clone()));
        }

        Ok(doc.to_writer(writer)?)
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

    pub fn read(reader: &mut dyn Read) -> Result<FileHeader, QoirDecodeError> {
        FileHeader::check_magic(reader)?;

        let doc = Document::from_reader(reader).map_err(err("Failed to read file header"))?;

        let (version, data_size, exif, width, height, channels, compression) = (
            doc.get_i32("version")
                .map_err(err("Failed to read file version"))? as u32,
            doc.get_i64("data_size")
                .map_err(err("Failed to read data size"))? as u64,
            doc.get_binary_generic("exif").ok().map(|b| b.to_vec()),
            doc.get_i32("width").map_err(err("Failed to read width"))? as u32,
            doc.get_i32("height")
                .map_err(err("Failed to read height"))? as u32,
            doc.get_i32("channels")
                .map_err(err("Failed to read channels"))? as u32,
            doc.get_i32("compression")
                .map_err(err("Failed to read compression"))? as u32,
        );

        Ok(Self {
            version,
            data_size,
            exif,
            width,
            height,
            channels: channels.try_into().map_err(err("Invalid channels"))?,
            compression: compression.try_into().map_err(err("Invalid compression"))?,
        })
    }
}

fn err<F>(e: &str) -> impl FnOnce(F) -> QoirDecodeError + '_ {
    |_| QoirDecodeError::InvalidFileHeader(e.to_string())
}
