# Koi - Kinda OK Image Format

A simple image format based on and inspired by [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html).

## Differences

- File header is encoded using BSON to allow for arbitrary metadata
- Streaming API for encoding and decoding with support for per-pixel read/write (for low memory devices)
- Support for grayscale images (with optional alpha channel) and gray delta encoding
- Support for alpha delta encoding (to reduce file size for transparency changes)
- LZ4 frame compression (instead of run-length encoding)

<!-- https://encode.su/threads/3753-QOI-(Quite-OK-Image-format)-lossless-image-compression-to-PNG-size -->
<!-- https://docs.rs/multiversion/latest/multiversion/ -->

## File format

### Header

To identify the file format, the first 8 bytes must be `KOI \xF0\x9F\x99\x82`. The file header is encoded using BSON. All numbers are u32 in i32 fields. The following fields are required:

- `v` (version): The file format version. Currently always `0`.
- `e` (exif): The EXIF data as a byte array.
- `w` (width): The image width in pixels. Must be greater than `0`.
- `h` (height): The image height in pixels. Must be greater than `0`.
- `c` (channels): The number of channels. Must be `1`, `2`, `3` or `4`.
- `x` (compression): The compression algorithm. Must be `0` (none) or `1` (LZ4 frame).

### Chunks

```
┌─ OP_INDEX ──────────────┐
│         Byte[0]         │
│  7  6  5  4  3  2  1  0 │
│───────┼─────────────────│
│  0  0 │      index      │
└───────┴─────────────────┘
6 bit index into the color index table (The table is only actually 62 entries long since this resulted in better compression with our hash function)
index_position = (r * 3 + g * 5 + b * 7 + a * 11) % 62

┌─ OP_DIFF ───────────────┐
│         Byte[0]         │
│  7  6  5  4  3  2  1  0 │
│───────┼─────────────────│
│  0  1 │ dr  │ dg  │ db  │
└───────┴─────────────────┘
Just like QOI (https://qoiformat.org/qoi-specification.pdf)


┌─ OP_DIFF_LUMA ──────────┬─ OP_DIFF_LUMA ──────────┐
│         Byte[0]         │         Byte[0]         │
│  7  6  5  4  3  2  1  0 │  7  6  5  4  3  2  1  0 │
│───────┼─────────────────│────────────┼────────────│
│  1  0 │   diff green    │   dr - dg  │  db - dg   │
└───────┴─────────────────┴────────────┴────────────┘
Also just like QOI (https://qoiformat.org/qoi-specification.pdf)


┌─ OP_DIFF_ALPHA ─────────┐
│         Byte[0]         │
│  7  6  5  4  3  2  1  0 │
│───────┼─────────────────│
│  1  1 │  alpha diff     │
└───────┴─────────────────┘
Note that 0xfc - 0xff are illegal for alpha diff because they are used for other opcodes.
```

### Ending

The file ends with these 8 bytes: `\x00\x00\x00\x00\xF0\x9F\x99\x82`.
