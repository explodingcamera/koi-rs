# Koi - Kinda OK Image Format

A simple image format based on and inspired by [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html).

## Differences

- File header is encoded using BSON to allow for arbitrary metadata
- Streaming API for encoding and decoding with support for per-pixel read/write (for low memory devices)
- Support for grayscale images (with optional alpha channel) and gray delta encoding
- Support for alpha delta encoding (to reduce file size for transparency changes)
- LZ4 frame compression (instead of run-length encoding)
- Improved hash function for color index table

A full overview of the differences and some benchmarks can be found in the accompanying [blog post](https://blog.henrygressmann.de/koi/).

<!-- https://encode.su/threads/3753-QOI-(Quite-OK-Image-format)-lossless-image-compression-to-PNG-size -->
<!-- https://docs.rs/multiversion/latest/multiversion/ -->

## Credits

- The [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html) formats
- A lot of great ideas from the [qoi2-bikeshed](https://github.com/nigeltao/qoi2-bikeshed/issues)

## Benchmarks

`cargo run --release --bin koi-bench`

### Findings

- Koi works great for images with lots of transparency and/or gradients like icons
- The encoding is not optimized yet and can be improved a lot
- Decode speed is close to PNG and outperforms it for small images
- Ratio wise Koi sits between the default and fast PNG compression levels, but often outperforms them in terms decode speed
- images with lots of detail can even be larger than the source
  - The hash function is not the best solution and misses a lot of obvious matches for images with lots of detail
- Koi generally outperforms PNG in terms of speed when using higher compression levels
- The new fast decoder for koi still needs some work, and can produce artifacts in some cases, but is already faster than png in many cases
