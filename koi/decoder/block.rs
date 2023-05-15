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

    let mut out = vec![0; header.width as usize * header.height as usize * C];
    let len = decode::<C>(&data, &mut out, header.clone())?;
    out.truncate(len);

    Ok(Image { header, data: out })
}

fn decode<const C: usize>(
    data: &[u8],
    out: &mut [u8],
    header: FileHeader,
) -> Result<usize, KoiDecodeError> {
    let mut data = Buffer::new(data);

    let out_buf_cap = out.len();
    let mut out_buf = BufferMut::new(out);

    let mut prev_pixel = Pixel::<C>::default();
    let mut out_chunk = [0; MAX_CHUNK_SIZE];

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

        if unlikely(len > MAX_CHUNK_SIZE as u32) {
            panic!("chunk too big");
        }

        let decompress_size =
            decompress(&data[..len as usize], &mut out_chunk, header.compression)?;
        data = data.advance(len as usize);

        let mut out_chunk_buf = BufferMut::new(&mut out_chunk[..decompress_size]);

        // iterate pixels times
        for _ in 0..pixels {
            let (buf, px) = decode_px::<C>(out_chunk_buf, prev_pixel);
            out_chunk_buf = buf;

            prev_pixel = px;
            out_buf = out_buf.write_many(&px.data);
        }
    }

    Ok(out_buf_cap - out_buf.len())
}

#[inline]
fn decode_px<'a, const C: usize>(
    buf: BufferMut<'a>,
    prev_pixel: Pixel<C>,
) -> (BufferMut<'a>, Pixel<C>) {
    match *buf {
        [OP_GRAY, v, ..] => {
            let px = Pixel::<C>::from_grayscale(v);
            (buf.advance(2), px)
        }
        [OP_GRAY_ALPHA, v, a, ..] => {
            let px = Pixel::<C>::from([v, v, v, a]);
            (buf.advance(3), px)
        }
        [OP_RGB, r, g, b, ..] => {
            let px = Pixel::<C>::from([r, g, b, 255]);
            (buf.advance(4), px)
        }

        [OP_RGBA, r, g, b, a, ..] => {
            let px = Pixel::<C>::from([r, g, b, a]);
            (buf.advance(5), px)
        }
        [b1 @ OP_DIFF_ALPHA..=OP_DIFF_ALPHA_END, ..] => {
            let px = prev_pixel.apply_alpha_diff(b1);
            (buf.advance(1), px)
        }
        [b1 @ OP_DIFF..=OP_DIFF_END, ..] => {
            let px = prev_pixel.apply_diff(b1);
            (buf.advance(1), px)
        }
        [b1 @ OP_LUMA..=OP_LUMA_END, b2, ..] => {
            let px = prev_pixel.apply_luma(b1, b2);
            (buf.advance(2), px)
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

pub fn decompress(
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
            let len = lz4_flex::block::decompress_into(&data, &mut out).map_err(|e| {
                println!("error: {}", e);
                KoiDecodeError::Decompress(e.to_string())
            })?;
            Ok(len)
        }
    }
}
