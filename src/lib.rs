use std::io::{BufReader, Read};
use std::io::{BufWriter, Write};
use thiserror::Error;

use lz4_flex::block::CompressError as Lz4CompressError;
use lz4_flex::block::DecompressError as Lz4DecompressError;
use lz4_flex::frame::Error as Lz4FrameError;
use types::RgbaColor;

pub mod file;
pub mod types;
pub mod util;
pub fn run() {}

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

pub fn qoir_decode(data: &mut dyn Read, writer: &mut dyn Write) -> Result<(), QoirDecodeError> {
    let reader = BufReader::new(data);
    let mut writer = BufWriter::new(writer);
    let mut index = [RgbaColor(0, 0, 0, 0); 64];

    let mut px: RgbaColor = RgbaColor(0, 0, 0, 0);

    Ok(())
}

pub fn qoir_encode(data: &mut dyn Read, writer: &mut dyn Write) -> Result<(), QoirEncodeError> {
    let reader = BufReader::new(data);
    let mut writer = BufWriter::new(writer);

    let mut index = [RgbaColor(0, 0, 0, 0); 64];
    let (mut px, mut px_prev) = (RgbaColor(0, 0, 0, 0), RgbaColor(0, 0, 0, 0));

    Ok(())
}
