# Koi - Kinda OK Image Format

A simple image format based on and inspired by [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html).

## Differences

- File header is encoded using BSON to allow for arbitrary metadata
- Streaming API for encoding and decoding with support for per-pixel read/write (for low memory devices)
- Support for grayscale images (with optional alpha channel) and gray delta encoding
- Support for alpha delta encoding (to reduce file size for transparency changes)
- LZ4 frame compression (instead of run-length encoding)
- Improved hash function for color index table

<!-- https://encode.su/threads/3753-QOI-(Quite-OK-Image-format)-lossless-image-compression-to-PNG-size -->
<!-- https://docs.rs/multiversion/latest/multiversion/ -->

## Credits

- The [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html) formats
- A lot of great ideas from the [qoi2-bikeshed](https://github.com/nigeltao/qoi2-bikeshed/issues)

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

## Benchmarks

`cargo run --release --bin koi-bench`

```
 Results

┌────────────────────────────────────┐
│ images/icon_512                    │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  2030ms │   127ms │  0.05 │
│ PngFast│   103ms │   156ms │  0.10 │
│ Koi    │   882ms │   155ms │  0.06 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/icon_64                     │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │    60ms │     6ms │  0.25 │
│ PngFast│     3ms │     5ms │  0.36 │
│ Koi    │    17ms │     4ms │  0.26 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/photo_kodak                 │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  1578ms │    83ms │  0.56 │
│ PngFast│    39ms │    56ms │  0.66 │
│ Koi    │   208ms │    76ms │  0.59 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/photo_tecnick               │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │ 27226ms │  1223ms │  0.55 │
│ PngFast│   604ms │   797ms │  0.58 │
│ Koi    │  3263ms │  1254ms │  0.60 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/photo_wikipedia             │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  9440ms │   488ms │  0.61 │
│ PngFast│   206ms │   301ms │  0.64 │
│ Koi    │  1232ms │   484ms │  0.66 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/pngimg                      │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │ 32196ms │  1551ms │  0.19 │
│ PngFast│   881ms │  1242ms │  0.22 │
│ Koi    │  5681ms │  1521ms │  0.20 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/screenshot_game             │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │ 44312ms │  1742ms │  0.22 │
│ PngFast│  1225ms │  1700ms │  0.28 │
│ Koi    │  7103ms │  1922ms │  0.24 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/screenshot_web              │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  4725ms │   342ms │  0.08 │
│ PngFast│   215ms │   336ms │  0.12 │
│ Koi    │  1786ms │   355ms │  0.07 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/textures_photo              │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  3494ms │   191ms │  0.57 │
│ PngFast│    73ms │   120ms │  0.82 │
│ Koi    │   468ms │   179ms │  0.65 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/textures_pk                 │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │   583ms │    51ms │  0.66 │
│ PngFast│    16ms │    26ms │  0.71 │
│ Koi    │    88ms │    35ms │  0.68 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/textures_pk01               │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  2484ms │   105ms │  0.33 │
│ PngFast│    62ms │    85ms │  0.44 │
│ Koi    │   307ms │    95ms │  0.34 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/textures_pk02               │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │ 18337ms │   544ms │  0.37 │
│ PngFast│   358ms │   481ms │  0.52 │
│ Koi    │  1611ms │   508ms │  0.41 │
└────────┴─────────┴─────────┴───────┘
┌────────────────────────────────────┐
│ images/textures_plants             │
├────────┬─────────┬─────────┬───────┤
│ format │ encode  │ decode  │ ratio │
├────────┼─────────┼─────────┼───────┤
│ Png    │  6773ms │   323ms │  0.23 │
│ PngFast│   152ms │   230ms │  0.24 │
│ Koi    │  1121ms │   303ms │  0.22 │
└────────┴─────────┴─────────┴───────┘
```

### Findings

- Koi works great for images with lots of transparency and/or gradients like icons
- The encoding is not optimized yet and can be improved a lot
- Decode speed is close to PNG and outperforms it for small images
- Ratio wise Koi sits between the default and fast PNG compression levels, but often outperforms them in terms decode speed
- images with lots of detail can even be larger than the source
  - The hash function is not the best solution and misses a lot of obvious matches for images with lots of detail
- Koi generally outperforms PNG in terms of speed when using higher compression levels
- The new fast decoder for koi still needs some work, and can produce artifacts in some cases, but is already faster than png in many cases
