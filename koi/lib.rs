use std::io::Write;

use file::FileHeader;
use thiserror::Error;

pub mod decoder;
pub mod encoder;
pub mod file;
pub mod types;
pub mod util;

pub fn encode<WRITER: std::io::Write, READER: std::io::Read, const CHANNELS: usize>(
    header: file::FileHeader,
    reader: READER, // unbuffered reader, if you want to use a buffered reader (e.g. when reading a file), wrap it in a BufReader
    mut writer: WRITER,
) -> Result<(), KoiEncodeError> {
    header.write(&mut writer)?;

    let mut encoder = match header.compression {
        types::Compression::None => encoder::PixelEncoder::<WRITER, CHANNELS>::new_uncompressed,
        types::Compression::Lz4 => encoder::PixelEncoder::<WRITER, CHANNELS>::new_lz4,
    }(writer, (header.width * header.height) as usize);

    encoder.encode(reader)?;
    encoder.flush()?;

    Ok(())
}

pub fn decode<WRITER: std::io::Write, READER: std::io::Read, const CHANNELS: usize>(
    mut reader: READER,
    mut writer: WRITER,
) -> Result<FileHeader, KoiDecodeError> {
    let header = file::FileHeader::read(&mut reader)?;

    let mut decoder = match header.compression {
        types::Compression::None => decoder::PixelDecoder::<READER, CHANNELS>::new_uncompressed,
        types::Compression::Lz4 => decoder::PixelDecoder::<READER, CHANNELS>::new_lz4,
    }(reader, (header.width * header.height) as usize);

    decoder.decode(&mut writer)?;
    Ok(header)
}

// pub fn decode_all<const C: usize>(
//     data: &[u8],
//     out: &mut [u8],
// ) -> Result<(FileHeader, usize), QoirDecodeError> {
//     let mut reader = std::io::Cursor::new(data);
//     let header = file::FileHeader::read(&mut reader)?;

//     let mut decoder =
//         decoder::PixelDecoder::<_, C>::new_lz4(reader, (header.width * header.height) as usize);
// }

#[derive(Error, Debug)]
pub enum KoiDecodeError {
    #[error("Invalid file header: {0}")]
    InvalidFileHeader(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("Failed to decompress: {0}")]
    Decompress(String),
}

#[derive(Error, Debug)]
pub enum KoiEncodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Bson(#[from] bson::ser::Error),

    #[error("Invalid length")]
    InvalidLength,

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),
}

#[derive(Error, Debug)]
pub enum KoiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    QoirEncodeError(#[from] KoiEncodeError),
    #[error(transparent)]
    QoirDecodeError(#[from] KoiDecodeError),
}

impl From<KoiEncodeError> for std::io::Error {
    fn from(err: KoiEncodeError) -> Self {
        match err {
            KoiEncodeError::Io(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, err),
        }
    }
}

impl From<KoiDecodeError> for std::io::Error {
    fn from(err: KoiDecodeError) -> Self {
        match err {
            KoiDecodeError::Io(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, err),
        }
    }
}
