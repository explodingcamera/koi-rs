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

pub fn encode<WRITER: std::io::Write, READER: std::io::Read, const CHANNELS: usize>(
    header: file::FileHeader,
    reader: READER, // unbuffered reader, if you want to use a buffered reader (e.g. when reading a file), wrap it in a BufReader
    mut writer: WRITER,
) -> Result<(), QoirEncodeError> {
    header.write(&mut writer)?;

    let mut encoder = encoder::PixelEncoder::<WRITER, CHANNELS>::new_lz4(
        writer,
        (header.width * header.height) as usize,
    );

    encoder.encode(reader)?;
    encoder.flush()?;

    Ok(())
}

pub fn decode<WRITER: std::io::Write, READER: std::io::Read, const CHANNELS: usize>(
    mut reader: READER,
    mut writer: WRITER,
) -> Result<FileHeader, QoirDecodeError> {
    let header = file::FileHeader::read(&mut reader)?;

    let mut decoder = decoder::PixelDecoder::<READER, CHANNELS>::new_lz4(
        reader,
        (header.width * header.height) as usize,
    );

    decoder.decode(&mut writer)?;
    Ok(header)
}

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
