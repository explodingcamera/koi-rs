use crate::{
    file::FileHeader,
    types::{color_diff, luma_diff, Channels, Op, Pixel},
    QoirEncodeError,
};

// has to be devisible by 1, 2, 3 and 4 so chunks_exact works properly
const CHUNK_SIZE: usize = 199992; // about 200kb

enum PixelEncoding {
    Index(u8),
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
                buf[0] = *data;
                1
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
    prev_pixel: Pixel<C>,
) -> PixelEncoding {
    // index encoding
    let hash = curr_pixel.hash();
    let index_px = cache[hash as usize];
    if index_px == prev_pixel {
        return PixelEncoding::Index(u8::from(Op::Index) | hash);
    }
    cache[hash as usize] = curr_pixel;

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
    let diff = curr_pixel.diff(&prev_pixel);

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
    if header.version != 1 {
        return Err(QoirEncodeError::UnsupportedVersion(header.version as u8));
    }

    let mut cache = [Pixel::default(); 256];
    let mut bytes_written = header.write_to_buf(out)?;
    let mut prev_pixel = Pixel::default();

    let mut out_chunk = [0; (CHUNK_SIZE * 4) / 3];
    for chunk in data.chunks(CHUNK_SIZE) {
        let mut chunk_bytes_written = 0;

        for px in chunk.chunks_exact(C) {
            // Safety: we know that px.len() == C, can be removed when array_exact is stable
            let curr_pixel = px.into();
            let encoded_px = encode_px::<C>(curr_pixel, &mut cache, prev_pixel);

            prev_pixel = curr_pixel;
            chunk_bytes_written += encoded_px.write_to_buf(&mut out_chunk[chunk_bytes_written..]);
        }

        let compress_size = lz4_flex::block::compress_into(
            out_chunk[..chunk_bytes_written].as_ref(),
            &mut out[bytes_written + 2..],
        )?;

        let prefix: [u8; 2] = (compress_size as u32).to_be_bytes()[..]
            .try_into()
            .map_err(|_| QoirEncodeError::InvalidLength)?;

        out[bytes_written..bytes_written + 2].copy_from_slice(&prefix);
        bytes_written += compress_size + 2;
    }

    Ok(bytes_written)
}
