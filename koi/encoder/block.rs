use crate::{
    types::{color_diff, luma_diff, Channels, Op, RgbaColor, CACHE_SIZE},
    util::pixel_hash,
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

#[inline]
fn encode_px<const CHANNELS: usize>(
    curr_pixel: RgbaColor,
    cache: &mut [RgbaColor; CACHE_SIZE],
    prev_pixel: &mut RgbaColor,
) -> PixelEncoding {
    // index encoding
    let hash = pixel_hash(curr_pixel);
    if cache[hash as usize] == curr_pixel {
        return PixelEncoding::Index([u8::from(Op::Index) | hash, 0]);
    }

    cache[hash as usize] = curr_pixel;

    // alpha diff encoding (whenever only alpha channel changes)
    if curr_pixel.0[..3] == prev_pixel.0[..3] {
        if let Some(diff) = prev_pixel.alpha_diff(&curr_pixel) {
            return PixelEncoding::AlphaDiff(diff);
        }
    }

    let is_gray = curr_pixel.is_gray();
    if curr_pixel.0[3] != prev_pixel.0[3] && curr_pixel.0[3] != 255 {
        // Gray Alpha encoding (whenever alpha channel changes and pixel is gray)
        if is_gray {
            return PixelEncoding::GrayAlpha([
                Op::GrayAlpha as u8,
                curr_pixel.0[0],
                curr_pixel.0[3],
            ]);
        }

        if CHANNELS != Channels::Rgba as u8 as usize {
            panic!("RGBA encoding is only supported for RGBA images");
        }

        // RGBA encoding (whenever alpha channel changes)
        return PixelEncoding::Rgba([
            Op::Rgba as u8,
            curr_pixel.0[0],
            curr_pixel.0[1],
            curr_pixel.0[2],
            curr_pixel.0[3],
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
        let RgbaColor([r, g, b, _]) = curr_pixel;
        if r == g && g == b {
            return PixelEncoding::Gray([Op::Gray as u8, curr_pixel.0[0]]);
        }
    }

    // RGB encoding
    PixelEncoding::Rgb([
        Op::Rgb as u8,
        curr_pixel.0[0],
        curr_pixel.0[1],
        curr_pixel.0[2],
    ])
}

pub fn encode<const CHANNELS: usize>(
    data: &[u8],
    out: &mut [u8],
) -> Result<usize, QoirEncodeError> {
    let mut cache = [RgbaColor([0, 0, 0, 0]); CACHE_SIZE];
    let mut bytes_written = 0;

    for chunk in data.chunks(CHUNK_SIZE) {
        let mut out_chunk = [0; CHUNK_SIZE];
        let mut chunk_bytes_written = 0;
        let mut prev_pixel = RgbaColor([0, 0, 0, 0]);
        for px in chunk.chunks_exact(CHANNELS) {
            let curr_pixel = RgbaColor::from_channels::<CHANNELS>(px);
            let encoded_px = encode_px::<CHANNELS>(curr_pixel, &mut cache, &mut prev_pixel);
            prev_pixel = curr_pixel;
            chunk_bytes_written += encoded_px.write_to_buf(&mut out_chunk[chunk_bytes_written..]);
        }

        bytes_written = lz4_flex::block::compress_into(
            out_chunk[..chunk_bytes_written].as_ref(),
            &mut out[bytes_written..],
        )?;
    }

    Ok(bytes_written)
}
