use crate::{
    file::FileHeader,
    types::{Channels, Op, Pixel, MAX_CHUNK_SIZE},
    util::{unlikely, BufferMut, Writer},
    KoiEncodeError,
};

// has to be devisible by 1, 2, 3 and 4 so chunks_exact works properly
const CHUNK_SIZE: usize = MAX_CHUNK_SIZE; // about 200kb

pub fn encode_to_vec<const CHANNELS: usize>(
    data: &[u8],
    header: FileHeader,
) -> Result<Vec<u8>, KoiEncodeError> {
    let mut out = vec![0; header.width as usize * header.height as usize * CHANNELS];
    let len = encode::<CHANNELS>(data, &mut out, header)?;
    out.truncate(len);

    Ok(out)
}

pub fn encode<const C: usize>(
    data: &[u8],
    out: &mut [u8],
    header: FileHeader,
) -> Result<usize, KoiEncodeError> {
    if header.version != 1 {
        return Err(KoiEncodeError::UnsupportedVersion(header.version as u8));
    }

    let out_buf_cap = out.len();
    let mut out_buf = BufferMut::new(out);
    out_buf = header.write_to_buf(out_buf)?;

    let mut prev_pixel = Pixel::default();

    const OUT_CHUNK_LEN: usize = CHUNK_SIZE * 2;
    let mut out_chunk = [0; OUT_CHUNK_LEN];

    for chunk in data.chunks(CHUNK_SIZE) {
        let mut out_chunk_buf = BufferMut::new(&mut out_chunk);
        let pixel_count = chunk.len() / C;

        for px in chunk.chunks_exact(C) {
            let px: [u8; C] = unsafe { px.try_into().unwrap_unchecked() };
            let curr_pixel = px.into();
            out_chunk_buf = encode_px::<C>(curr_pixel, prev_pixel, out_chunk_buf);
            prev_pixel = curr_pixel;
        }

        let bytes_written = OUT_CHUNK_LEN - out_chunk_buf.len();

        let compress_size = compress(
            &out_chunk[..bytes_written],
            &mut out_buf[8..],
            CompressionLevel::Lz4Hc(8),
        )?;

        let bytes_length: &[u8; 4] = &(compress_size as u32).to_le_bytes();
        let bytes_pixels: &[u8; 4] = &(pixel_count as u32).to_le_bytes();

        out_buf = out_buf.write_many(bytes_length);
        out_buf = out_buf.write_many(bytes_pixels);
        out_buf = out_buf.advance(compress_size);
    }

    Ok(out_buf_cap - out_buf.len())
}

pub enum CompressionLevel {
    Lz4Flex,
    Lz4(i32),
    Lz4Hc(i32),
    None,
}

pub fn compress(
    input: &[u8],
    mut output: &mut [u8],
    level: CompressionLevel,
) -> Result<usize, KoiEncodeError> {
    let out_size = match level {
        CompressionLevel::Lz4(level) => {
            lzzzz::lz4::compress(&input, &mut output, level).map_err(|e| {
                println!("error: {}", e);
                KoiEncodeError::InvalidLength
            })?
        }

        CompressionLevel::Lz4Hc(level) => lzzzz::lz4_hc::compress(&input, &mut output, level)
            .map_err(|e| {
                println!("error: {}", e);
                KoiEncodeError::InvalidLength
            })?,

        CompressionLevel::Lz4Flex => lz4_flex::compress_into(&input, &mut output).map_err(|e| {
            println!("error: {}", e);
            KoiEncodeError::InvalidLength
        })?,
        CompressionLevel::None => {
            output[..input.len()].copy_from_slice(input);
            input.len()
        }
    };

    Ok(out_size)
}

#[inline]
fn encode_px<'a, const C: usize>(
    curr_pixel: Pixel<C>,
    prev_pixel: Pixel<C>,
    buf: BufferMut<'a>,
) -> BufferMut<'a> {
    if (C == 2 || C == 4) && curr_pixel.rgb() == prev_pixel.rgb() {
        if let Some(diff) = prev_pixel.alpha_diff(&curr_pixel) {
            return buf.write_one(diff);
        }
    }

    let is_gray = curr_pixel.is_gray();
    if C != 1 && curr_pixel.a() != prev_pixel.a() && curr_pixel.a() != 255 {
        if is_gray {
            return buf.write_many(&[Op::GrayAlpha as u8, curr_pixel.r(), curr_pixel.a()]);
        }

        if unlikely(C != Channels::Rgba as u8 as usize) {
            panic!("RGBA encoding is currently only supported for RGBA images");
        }

        // RGBA encoding (whenever alpha channel changes)
        return buf.write_many(&[
            Op::Rgba as u8,
            curr_pixel.r(),
            curr_pixel.g(),
            curr_pixel.b(),
            curr_pixel.a(),
        ]);
    }

    // Difference between current and previous pixel
    let diff = curr_pixel.diff(&prev_pixel);
    let color_diff = diff.color();

    // Diff encoding
    if let Some(diff) = color_diff {
        return buf.write_one(diff);
    }

    // Luma encoding
    if let Some(luma) = diff.luma() {
        return buf.write_many(&luma);
    }

    if is_gray {
        // Gray encoding
        return buf.write_many(&[Op::Gray as u8, curr_pixel.r()]);
    }

    buf.write_many(&[
        Op::Rgb as u8,
        curr_pixel.r(),
        curr_pixel.g(),
        curr_pixel.b(),
    ])
}
