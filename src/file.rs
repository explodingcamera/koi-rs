use std::io::{Error, ErrorKind, Read, Write};

use bson::{Binary, Document};

use crate::{QoirDecodeError, QoirEncodeError};

// magic number to identify koi files (KOI + 🙂|🙃)
const MAGIC: &[u8] = b"KOI\xF0\x9F\x99\x82|\xF0\x9F\x99\x83";

#[derive(Debug)]
pub struct FileHeader {
    version: u32,
    data_size: u64,
    exif: Option<Vec<u8>>,
}

#[inline]
fn to_binary(bytes: Vec<u8>) -> Binary {
    Binary {
        bytes,
        subtype: bson::spec::BinarySubtype::Generic,
    }
}

impl FileHeader {
    pub fn new(data_size: u64, exif: Option<Vec<u8>>) -> FileHeader {
        FileHeader {
            version: 0,
            data_size,
            exif,
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

        let doc = Document::from_reader(reader).map_err(|_| {
            QoirDecodeError::InvalidFileHeader("Failed to read file header".to_string())
        })?;

        let version = doc.get_i32("version").map_err(|_| {
            QoirDecodeError::InvalidFileHeader("Failed to read file version".to_string())
        })?;

        let data_size = doc.get_i64("data_size").map_err(|_| {
            QoirDecodeError::InvalidFileHeader("Failed to read data size".to_string())
        })? as u64;

        let exif = doc.get_binary_generic("exif").ok().map(|b| b.to_vec());

        Ok(Self {
            version: version as u32,
            data_size: data_size as u64,
            exif,
        })
    }
}
