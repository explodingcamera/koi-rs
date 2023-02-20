use std::io::{Read, Write};

use bson::{Binary, Document};

use crate::{QoirDecodeError, QoirEncodeError};

pub struct FileHeader {
    version: u32,
    data_size: u64,
    exif: Option<Vec<u8>>,
}

impl FileHeader {
    fn new(size: u64) -> FileHeader {
        FileHeader {
            version: 0,
            data_size: size,
            exif: None,
        }
    }

    pub fn write(&self, writer: &mut dyn Write) -> Result<(), QoirEncodeError> {
        let mut doc = Document::new();
        doc.insert("version", self.version as i32);
        doc.insert("data_size", self.data_size as i64);

        if let Some(exif) = &self.exif {
            doc.insert(
                "exif",
                Binary {
                    bytes: exif.clone(),
                    subtype: bson::spec::BinarySubtype::Generic,
                },
            );
        }

        doc.to_writer(writer).map_err(|_| {
            QoirEncodeError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write file header",
            ))
        })?;

        Ok(())
    }

    pub fn read(reader: &mut dyn Read) -> Result<FileHeader, QoirDecodeError> {
        let doc = Document::from_reader(reader).map_err(|_| QoirDecodeError::InvalidFileHeader)?;

        let version = doc
            .get_i32("version")
            .map_err(|_| QoirDecodeError::InvalidFileHeader)?;

        let data_size = doc
            .get_i64("data_size")
            .map_err(|_| QoirDecodeError::InvalidFileHeader)?;

        let exif = doc.get_binary_generic("exif").ok().map(|b| b.to_vec());

        Ok(Self {
            version: version as u32,
            data_size: data_size as u64,
            exif,
        })
    }
}
