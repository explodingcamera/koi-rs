use crate::{
    file::FileHeader,
    types::{Channels, Op, Pixel},
    util::{unlikely, Buffer, Writer},
    QoirEncodeError,
};

// has to be devisible by 1, 2, 3 and 4 so chunks_exact works properly
const CHUNK_SIZE: usize = 199992; // about 200kb

// enum PixelEncoding {
//     Diff(u8),
//     AlphaDiff(u8),
//     LumaDiff([u8; 2]),
//     Gray([u8; 2]),
//     GrayAlpha([u8; 3]),
//     Rgba([u8; 5]),
//     Rgb([u8; 4]),
// }

// impl PixelEncoding {
//     #[inline]
//     fn write_to_buf(&self, buf: Buffer) -> Buffer {
//         match self {
//             PixelEncoding::Diff(data) => buf.write_one(*data),
//             PixelEncoding::AlphaDiff(data) => buf.write_one(*data),
//             PixelEncoding::LumaDiff(data) => buf.write_many(data),
//             PixelEncoding::Gray(data) => buf.write_many(data),
//             PixelEncoding::GrayAlpha(data) => buf.write_many(data),
//             PixelEncoding::Rgba(data) => buf.write_many(data),
//             PixelEncoding::Rgb(data) => buf.write_many(data),
//         }
//     }
// }

#[inline]
fn encode_px<'a, const C: usize>(
    curr_pixel: Pixel<C>,
    prev_pixel: Pixel<C>,
    buf: Buffer<'a>,
) -> Buffer<'a> {
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
        return buf.write_many(&curr_pixel.rgba());
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

pub fn encode_to_vec<const CHANNELS: usize>(
    data: &[u8],
    header: FileHeader,
) -> Result<Vec<u8>, QoirEncodeError> {
    let mut out = vec![0; header.width as usize * header.height as usize * CHANNELS];
    let len = encode::<CHANNELS>(data, &mut out, header)?;
    out.truncate(len);

    Ok(out)
}

pub fn encode<const C: usize>(
    data: &[u8],
    out: &mut [u8],
    header: FileHeader,
) -> Result<usize, QoirEncodeError> {
    if header.version != 1 {
        return Err(QoirEncodeError::UnsupportedVersion(header.version as u8));
    }

    let mut out_buf = Buffer::new(out);
    let out_buf_cap = out_buf.capacity();
    out_buf = header.write_to_buf(out_buf)?;

    let mut prev_pixel = Pixel::default();

    const OUT_CHUNK_LEN: usize = CHUNK_SIZE * 2;
    let mut out_chunk = [0; OUT_CHUNK_LEN];

    for chunk in data.chunks(CHUNK_SIZE) {
        let mut out_chunk_buf = Buffer::new(&mut out_chunk);

        for px in chunk.chunks_exact(C) {
            let px: [u8; C] = unsafe { px.try_into().unwrap_unchecked() };
            let curr_pixel = px.into();
            out_chunk_buf = encode_px::<C>(curr_pixel, prev_pixel, out_chunk_buf);
            prev_pixel = curr_pixel;
        }

        let bytes_written = OUT_CHUNK_LEN - out_chunk_buf.len();

        let compress_size = lz4_flex::compress_into(&out_chunk[..bytes_written], &mut out_buf)
            .map_err(|e| {
                println!("error: {}", e);
                QoirEncodeError::InvalidLength
            })?;
        // lz4::block::compress_to_buffer(&out_chunk[..bytes_written], None, true, &mut out_buf)
        //     .map_err(|e| {
        //     println!("error: {}", e);
        //     QoirEncodeError::InvalidLength
        // })?;
        // lzzzz::lz4_hc::compress(
        //     &out_chunk[..bytes_written],
        //     &mut out_buf,
        //     12,
        // ).map_err(|e| {
        //     println!("error: {}", e);
        //     QoirEncodeError::InvalidLength
        // })?;
        // lzzzz::lz4::compress(
        //     &out_chunk[..bytes_written],
        //     &mut out_buf,
        //     12,
        // ).map_err(|e| {
        //     println!("error: {}", e);
        //     QoirEncodeError::InvalidLength
        // })?;

        out_buf = out_buf.trim_start(compress_size);
    }

    Ok(out_buf_cap - out_buf.len())
}
