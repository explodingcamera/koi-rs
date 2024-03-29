use crate::{
    file::FileHeader,
    types::*,
    util::{cold, unlikely, Buffer, BufferMut, Writer},
    KoiDecodeError,
};

pub struct Image {
    pub header: FileHeader,
    pub data: Vec<u8>,
}

pub fn decode_to_vec<const C: usize>(data: &[u8]) -> Result<Image, KoiDecodeError> {
    let data = Buffer::new(data);
    let (data, header) = FileHeader::read_buf(data)?;

    if header.version != 1 {
        return Err(KoiDecodeError::UnsupportedVersion(header.version as u8));
    }

    let mut out = vec![0; header.min_output_size()];
    let len = decode_impl::<C>(&data, &mut out, header.clone())?;
    out.truncate(len);

    Ok(Image { header, data: out })
}

pub fn min_output_size<const C: usize>(data: &[u8]) -> usize {
    FileHeader::read_bytes(data).unwrap().1.min_output_size()
}

pub fn decode<const C: usize>(
    data: &[u8],
    out: &mut [u8],
) -> Result<(usize, FileHeader), KoiDecodeError> {
    let data = Buffer::new(data);
    let (data, header) = FileHeader::read_buf(data)?;

    if header.version != 1 {
        return Err(KoiDecodeError::UnsupportedVersion(header.version as u8));
    }

    let len = decode_impl::<C>(&data, out, header.clone())?;
    Ok((len, header))
}

fn decode_impl<const C: usize>(
    data: &[u8],
    out: &mut [u8],
    header: FileHeader,
) -> Result<usize, KoiDecodeError> {
    let mut data = Buffer::new(data);

    let out_buf_cap = out.len();
    let mut out_buf = BufferMut::new(out);

    let mut prev_pixel = Pixel::<C>::default();
    let mut out_chunk = [0; (MAX_CHUNK_SIZE * 2)];

    loop {
        if data.is_empty() {
            break;
        }

        let len: u32;
        let pixels: u32;
        (len, data) = data.read_u32_le();
        (pixels, data) = data.read_u32_le();

        if len == 0 {
            break;
        }

        if unlikely(len as usize > (MAX_CHUNK_SIZE * 2)) {
            panic!("chunk too big: {}", len);
        }

        let decompress_size =
            decompress(&data[..len as usize], &mut out_chunk, header.compression)?;
        data = data.advance(len as usize);

        let mut out_chunk_buf = &mut out_chunk[..decompress_size];

        // iterate pixels times
        for _ in 0..pixels {
            let px: Pixel<C>;
            (out_chunk_buf, px) = decode_px::<C>(out_chunk_buf, prev_pixel);

            prev_pixel = px;
            out_buf = out_buf.write_many(&px.data);
        }
    }

    Ok(out_buf_cap - out_buf.len())
}

#[allow(clippy::all)] // clippy is making the code slower
fn decode_px<'a, const C: usize>(
    data: &'a mut [u8],
    prev_pixel: Pixel<C>,
) -> (&'a mut [u8], Pixel<C>) {
    match data {
        [OP_SAME, rest @ ..] => (rest, prev_pixel),
        [OP_GRAY, v, rest @ ..] => (rest, Pixel::<C>::from_grayscale(*v)),
        [OP_GRAY_ALPHA, v, a, rest @ ..] => (rest, Pixel::<C>::from([*v, *v, *v, *a])),
        [OP_RGB, r, g, b, rest @ ..] => (rest, Pixel::<C>::from([*r, *g, *b, 255])),
        [OP_RGBA, r, g, b, a, rest @ ..] => (rest, Pixel::<C>::from([*r, *g, *b, *a])),

        [b1 @ OP_DIFF..=OP_DIFF_END, rest @ ..] => (rest, prev_pixel.apply_diff(*b1)),
        [b1 @ OP_LUMA..=OP_LUMA_END, b2, rest @ ..] => (rest, prev_pixel.apply_luma(*b1, *b2)),
        [b1 @ OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END, rest @ ..] => {
            (rest, prev_pixel.apply_alpha_diff(*b1))
        }

        [opcode, ..] => {
            cold();
            panic!("Invalid opcode {}", opcode);
        }
        _ => {
            cold();
            panic!("Invalid opcode");
        }
    }
}

#[allow(clippy::all)] // clippy is making the code slower
fn decompress(
    data: &[u8],
    mut out: &mut [u8],
    compression: Compression,
) -> Result<usize, KoiDecodeError> {
    match compression {
        Compression::None => {
            out[..data.len()].copy_from_slice(data);
            Ok(data.len())
        }
        Compression::Lz4 => {
            // let len = lz4_flex::block::decompress_into(&data, &mut out).map_err(|e| {
            //     println!("error: {}", e);
            //     KoiDecodeError::Decompress(e.to_string())
            // })?;
            // Ok(len)

            // lzzz is slightly faster than lz4_flex, but not portable
            let len = lzzzz::lz4::decompress(&data, &mut out).map_err(|e| {
                println!("error: {}", e);
                KoiDecodeError::Decompress(e.to_string())
            })?;
            Ok(len)
        }
    }
}
