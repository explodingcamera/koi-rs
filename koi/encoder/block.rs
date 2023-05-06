use core::slice;
use std::io::Cursor;

use bytes::{BufMut, BytesMut};

use crate::{
    file::FileHeader,
    types::{color_diff, luma_diff, Channels, Op, Pixel, CACHE_SIZE},
    QoirEncodeError,
};

// has to be devisible by 1, 2, 3 and 4 so chunks_exact works properly
// less than 64k tp fit into u16
const CHUNK_SIZE: usize = 64 * 1024 - 4;

enum PixelEncoding {
    Index([u8; 2]),
    Diff(u8),
    AlphaDiff(u8),
    LumaDiff([u8; 2]),
    Gray([u8; 2]),
    GrayAlpha([u8; 3]),
    Rgba([u8; 5]),
    Rgb([u8; 4]),
}

impl PixelEncoding {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self {
            PixelEncoding::Index(data) => data,
            PixelEncoding::Diff(data) => slice::from_ref(data),
            PixelEncoding::AlphaDiff(data) => slice::from_ref(data),
            PixelEncoding::LumaDiff(data) => data,
            PixelEncoding::Gray(data) => data,
            PixelEncoding::GrayAlpha(data) => data,
            PixelEncoding::Rgba(data) => data,
            PixelEncoding::Rgb(data) => data,
        }
    }

    #[inline]
    fn write_to_bytes(&self, bytes: &mut BytesMut) {
        match self {
            PixelEncoding::Index(data) => {
                bytes.put(&data[..]);
            }
            PixelEncoding::Diff(data) => {
                bytes.put_u8(*data);
            }
            PixelEncoding::AlphaDiff(data) => {
                bytes.put_u8(*data);
            }
            PixelEncoding::LumaDiff(data) => {
                bytes.put(&data[..]);
            }
            PixelEncoding::Gray(data) => {
                bytes.put(&data[..]);
            }
            PixelEncoding::GrayAlpha(data) => {
                bytes.put(&data[..]);
            }
            PixelEncoding::Rgba(data) => {
                bytes.put(&data[..]);
            }
            PixelEncoding::Rgb(data) => {
                bytes.put(&data[..]);
            }
        }
    }

    #[inline]
    fn write_to_buf(&self, buf: &mut [u8]) -> usize {
        match self {
            PixelEncoding::Index(data) => {
                buf[..2].copy_from_slice(data);
                2
            }
            PixelEncoding::Diff(data) => {
                buf[0] = *data;
                1
            }
            PixelEncoding::AlphaDiff(data) => {
                buf[0] = *data;
                1
            }
            PixelEncoding::LumaDiff(data) => {
                buf[..2].copy_from_slice(data);
                2
            }
            PixelEncoding::Gray(data) => {
                buf[..2].copy_from_slice(data);
                2
            }
            PixelEncoding::GrayAlpha(data) => {
                buf[..3].copy_from_slice(data);
                3
            }
            PixelEncoding::Rgba(data) => {
                buf[..5].copy_from_slice(data);
                5
            }
            PixelEncoding::Rgb(data) => {
                buf[..4].copy_from_slice(data);
                4
            }
        }
    }
}

#[inline(always)]
fn encode_px<const C: usize>(
    curr_pixel: Pixel<C>,
    cache: &mut [Pixel<C>; 256],
    prev_pixel: &mut Pixel<C>,
) -> PixelEncoding {
    // index encoding
    let hash_prev = prev_pixel.hash();
    let index_px = cache[hash_prev as usize];
    if index_px == curr_pixel {
        return PixelEncoding::Index([u8::from(Op::Index) | hash_prev, 0]);
    }
    cache[hash_prev as usize] = curr_pixel;

    // alpha diff encoding (whenever only alpha channel changes)
    if C > 3 && curr_pixel.a() == prev_pixel.a() {
        if let Some(diff) = prev_pixel.alpha_diff(&curr_pixel) {
            return PixelEncoding::AlphaDiff(diff);
        }
    }

    let is_gray = curr_pixel.is_gray();
    if C != 1 && curr_pixel.a() != prev_pixel.a() && curr_pixel.a() != 255 {
        // Gray Alpha encoding (whenever alpha channel changes and pixel is gray)
        if is_gray {
            return PixelEncoding::GrayAlpha([Op::GrayAlpha as u8, curr_pixel.r(), curr_pixel.a()]);
        }

        if C != Channels::Rgba as u8 as usize {
            panic!("RGBA encoding is only supported for RGBA images");
        }

        // RGBA encoding (whenever alpha channel changes)
        return PixelEncoding::Rgba([
            Op::Rgba as u8,
            curr_pixel.r(),
            curr_pixel.g(),
            curr_pixel.b(),
            curr_pixel.a(),
        ]);
    }

    // Difference between current and previous pixel
    let diff = curr_pixel.diff(prev_pixel);

    // Diff encoding
    if let Some(diff) = color_diff(diff) {
        return PixelEncoding::Diff(diff);
    }

    // Luma encoding (a little bit broken on fast_decode) TODO: fix
    if let Some(luma) = luma_diff(diff) {
        return PixelEncoding::LumaDiff(luma);
    }

    if is_gray {
        // Gray encoding
        return PixelEncoding::Gray([Op::Gray as u8, curr_pixel.r()]);
    }

    // RGB encoding
    PixelEncoding::Rgb([
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
    let bytes_written = encode::<CHANNELS>(data, &mut out, header)?;
    out.truncate(bytes_written);
    Ok(out)
}

pub fn encode<const C: usize>(
    data: &[u8],
    out: &mut [u8],
    header: FileHeader,
) -> Result<usize, QoirEncodeError> {
    let mut cache = [Pixel::default(); 256];
    let mut bytes_written = header.write_to_buf(out)?;
    let mut prev_pixel = Pixel::default();

    let mut out_chunk = [0; CHUNK_SIZE];
    for chunk in data.chunks(CHUNK_SIZE) {
        let mut chunk_bytes_written = 0;

        for px in chunk.chunks_exact(C) {
            let px = unsafe { px.try_into().unwrap_unchecked() };
            let curr_pixel = Pixel::from_channels::<C>(px);
            let encoded_px = encode_px::<C>(curr_pixel, &mut cache, &mut prev_pixel);

            prev_pixel = curr_pixel;
            chunk_bytes_written += encoded_px.write_to_buf(&mut out_chunk[chunk_bytes_written..]);
        }

        // for px in  {
        //     let curr_pixel = RgbaColor::from_channels::<CHANNELS>(px);
        //     let encoded_px = encode_px::<CHANNELS>(curr_pixel, &mut cache, &mut prev_pixel);
        //     prev_pixel = curr_pixel;
        //     // chunk_bytes_written += encoded_px.write_to_buf(&mut out_chunk[chunk_bytes_written..]);
        //     out_chunk
        //         .extend_from_slice(encoded_px.write_to_buf(&mut out_chunk[chunk_bytes_written..]));
        // }

        bytes_written += lz4_flex::block::compress_into(
            out_chunk[..chunk_bytes_written].as_ref(),
            &mut out[bytes_written..],
        )?;
    }

    Ok(bytes_written)
}
