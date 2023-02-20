use std::io::{BufReader, Read};
use std::io::{BufWriter, Write};
use thiserror::Error;

use lz4_flex::block::CompressError as Lz4CompressError;
use lz4_flex::block::DecompressError as Lz4DecompressError;
use lz4_flex::frame::Error as Lz4FrameError;

mod file;

pub fn run() {}

#[derive(Error, Debug)]
pub enum QoirDecodeError {
    #[error("Invalid file header")]
    InvalidFileHeader,

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
    Lz4Frame(#[from] Lz4FrameError),

    #[error(transparent)]
    Lz4Compress(#[from] Lz4CompressError),
}

// take a reader and return a vector of bytes
pub fn qoir_decode(data: &mut dyn Read) -> Result<Vec<u8>, QoirDecodeError> {
    let buffer = BufReader::new(data);
    let mut result = Vec::new();

    Ok(result)
}

// take a array of bytes and a writer
pub fn qoir_encode(data: &[u8], writer: &mut dyn Write) -> Result<(), QoirEncodeError> {
    let buffer = BufWriter::new(writer);

    Ok(())
}
