use std::io::Write;

use file::FileHeader;
use thiserror::Error;

use lz4_flex::block::CompressError as Lz4CompressError;
use lz4_flex::block::DecompressError as Lz4DecompressError;
use lz4_flex::frame::Error as Lz4FrameError;

pub mod decoder;
pub mod encoder;
pub mod file;
pub mod types;
pub mod util;
pub fn run() {}

pub fn encode<
    WRITER: std::io::Write,
    READER: std::io::Read,
    const CHANNELS: usize,
    const COMPRESSION: u8,
>(
    header: file::FileHeader,
    reader: READER, // unbuffered reader, if you want to use a buffered reader (e.g. when reading a file), wrap it in a BufReader
    mut writer: WRITER,
) -> Result<(), QoirEncodeError> {
    header.write(&mut writer)?;

    let mut encoder = match COMPRESSION {
        0 => encoder::PixelEncoder::<WRITER, CHANNELS>::new_uncompressed,
        1 => encoder::PixelEncoder::<WRITER, CHANNELS>::new_lz4,
        _ => panic!("Invalid compression type. Valid values are 0 (uncompressed) and 1 (lz4)"),
    }(writer, (header.width * header.height) as usize);

    encoder.encode(reader)?;
    encoder.flush()?;

    Ok(())
}

pub fn decode<WRITER: std::io::Write, READER: std::io::Read, const CHANNELS: usize>(
    mut reader: READER,
    mut writer: WRITER,
) -> Result<FileHeader, QoirDecodeError> {
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
pub enum QoirDecodeError {
    #[error("Invalid file header: {0}")]
    InvalidFileHeader(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Lz4Frame(#[from] Lz4FrameError),

    #[error(transparent)]
    Lz4Decompress(#[from] Lz4DecompressError),
}

#[derive(Error, Debug)]
pub enum QoirEncodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Bson(#[from] bson::ser::Error),

    #[error(transparent)]
    Lz4Frame(#[from] Lz4FrameError),

    #[error(transparent)]
    Lz4Compress(#[from] Lz4CompressError),
}

impl From<QoirEncodeError> for std::io::Error {
    fn from(err: QoirEncodeError) -> Self {
        match err {
            QoirEncodeError::Io(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, err),
        }
    }
}

impl From<QoirDecodeError> for std::io::Error {
    fn from(err: QoirDecodeError) -> Self {
        match err {
            QoirDecodeError::Io(err) => err,
            _ => std::io::Error::new(std::io::ErrorKind::Other, err),
        }
    }
}
